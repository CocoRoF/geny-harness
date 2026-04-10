"""Stage 6: API — calls Anthropic Messages API.

This is the core execution stage.  It uses the ``anthropic`` Python SDK
to make real API calls, supporting both non-streaming and streaming modes.
When streaming, it emits ``text.delta`` events through ``state.add_event``
so that ``Pipeline.run_stream()`` can yield them to the web UI in
real-time.
"""

from __future__ import annotations

import asyncio
from typing import Any, AsyncIterator, Dict, List, Optional

from geny_harness._native import (
    APIError,
    ErrorCategory,
    PipelineState,
    TokenUsage,
)
from geny_harness.core.stage import Stage, StrategyInfo


# ---------------------------------------------------------------------------
# Anthropic provider (lazy-import of the SDK)
# ---------------------------------------------------------------------------

class AnthropicProvider:
    """Real Anthropic API provider using the official SDK."""

    def __init__(
        self,
        api_key: str,
        base_url: Optional[str] = None,
        default_headers: Optional[Dict[str, str]] = None,
    ):
        self._api_key = api_key
        self._base_url = base_url
        self._default_headers = default_headers
        self._client: Optional[Any] = None

    def _get_client(self) -> Any:
        if self._client is None:
            import anthropic

            kwargs: Dict[str, Any] = {"api_key": self._api_key}
            if self._base_url:
                kwargs["base_url"] = self._base_url
            if self._default_headers:
                kwargs["default_headers"] = self._default_headers
            self._client = anthropic.AsyncAnthropic(**kwargs)
        return self._client

    # -- non-streaming --

    async def create_message(self, kwargs: Dict[str, Any]) -> Any:
        """Call messages.create and return the raw SDK response object."""
        client = self._get_client()
        try:
            return await client.messages.create(**kwargs)
        except Exception as e:
            raise self._classify_error(e) from e

    # -- streaming --

    async def create_message_stream(
        self, kwargs: Dict[str, Any]
    ) -> AsyncIterator[Dict[str, Any]]:
        """Streaming call.  Yields ``text_delta`` dicts then ``message_complete``."""
        client = self._get_client()
        try:
            async with client.messages.stream(**kwargs) as stream:
                async for text in stream.text_stream:
                    yield {"type": "text_delta", "text": text}

                final = await stream.get_final_message()
                yield {"type": "message_complete", "response": final}
        except Exception as e:
            raise self._classify_error(e) from e

    # -- error mapping --

    @staticmethod
    def _classify_error(e: Exception) -> APIError:
        import anthropic

        if isinstance(e, anthropic.RateLimitError):
            return APIError(str(e), category=ErrorCategory.RATE_LIMITED, cause=e)
        if isinstance(e, anthropic.APITimeoutError):
            return APIError(str(e), category=ErrorCategory.TIMEOUT, cause=e)
        if isinstance(e, anthropic.APIConnectionError):
            return APIError(str(e), category=ErrorCategory.NETWORK, cause=e)
        if isinstance(e, anthropic.AuthenticationError):
            return APIError(str(e), category=ErrorCategory.AUTH, status_code=401, cause=e)
        if isinstance(e, anthropic.BadRequestError):
            msg = str(e).lower()
            if "token" in msg or "context" in msg:
                return APIError(str(e), category=ErrorCategory.TOKEN_LIMIT, status_code=400, cause=e)
            return APIError(str(e), category=ErrorCategory.BAD_REQUEST, status_code=400, cause=e)
        if isinstance(e, anthropic.InternalServerError):
            return APIError(str(e), category=ErrorCategory.SERVER_ERROR, status_code=500, cause=e)
        if isinstance(e, APIError):
            return e
        return APIError(str(e), category=ErrorCategory.UNKNOWN, cause=e)


# ---------------------------------------------------------------------------
# Response helpers
# ---------------------------------------------------------------------------

def _parse_raw_response(raw: Any) -> Dict[str, Any]:
    """Convert a raw Anthropic SDK Message into a simple dict structure."""
    text_parts: List[str] = []
    tool_calls: List[Dict[str, Any]] = []
    thinking_texts: List[str] = []
    content_blocks: List[Dict[str, Any]] = []

    for block in raw.content:
        if block.type == "text":
            text_parts.append(block.text)
            content_blocks.append({"type": "text", "text": block.text})
        elif block.type == "tool_use":
            tool_calls.append({
                "tool_use_id": block.id,
                "tool_name": block.name,
                "tool_input": block.input,
            })
            content_blocks.append({
                "type": "tool_use",
                "id": block.id,
                "name": block.name,
                "input": block.input,
            })
        elif block.type == "thinking":
            thinking_texts.append(block.thinking)

    usage = TokenUsage(
        input_tokens=getattr(raw.usage, "input_tokens", 0),
        output_tokens=getattr(raw.usage, "output_tokens", 0),
        cache_creation_input_tokens=getattr(raw.usage, "cache_creation_input_tokens", 0),
        cache_read_input_tokens=getattr(raw.usage, "cache_read_input_tokens", 0),
    )

    return {
        "text": "".join(text_parts),
        "tool_calls": tool_calls,
        "thinking_texts": thinking_texts,
        "content_blocks": content_blocks,
        "stop_reason": raw.stop_reason or "",
        "usage": usage,
        "model": raw.model,
        "message_id": raw.id,
    }


# ---------------------------------------------------------------------------
# APIStage
# ---------------------------------------------------------------------------

class APIStage(Stage[Any, Dict[str, Any]]):
    """Stage 6: API.

    Calls the Anthropic Messages API and stores the parsed response in
    ``state.last_api_response``.  When ``stream=True`` (default), emits
    ``text.delta`` events for real-time token streaming.
    """

    def __init__(
        self,
        *,
        api_key: str = "",
        base_url: Optional[str] = None,
        stream: bool = True,
        max_retries: int = 2,
        provider: Optional[AnthropicProvider] = None,
    ):
        if provider:
            self._provider = provider
        elif api_key:
            self._provider = AnthropicProvider(api_key=api_key, base_url=base_url)
        else:
            raise ValueError("Either 'provider' or 'api_key' must be provided")

        self._stream = stream
        self._max_retries = max_retries

    @property
    def name(self) -> str:
        return "api"

    @property
    def order(self) -> int:
        return 6

    @property
    def category(self) -> str:
        return "execution"

    async def execute(self, input: Any, state: PipelineState) -> Dict[str, Any]:
        kwargs = self._build_kwargs(state)

        state.add_event(
            "api.request",
            {
                "model": kwargs.get("model", ""),
                "message_count": len(kwargs.get("messages", [])),
                "has_tools": bool(kwargs.get("tools")),
                "stream": self._stream,
            },
        )

        if self._stream:
            parsed = await self._call_streaming(kwargs, state)
        else:
            parsed = await self._call_with_retry(kwargs, state)

        # Store for downstream stages
        state.last_api_response = parsed

        # Build assistant message content for conversation history
        assistant_content = self._build_assistant_content(parsed)
        state.add_message("assistant", assistant_content)

        state.add_event(
            "api.response",
            {
                "stop_reason": parsed["stop_reason"],
                "text_length": len(parsed["text"]),
                "tool_calls": len(parsed["tool_calls"]),
                "input_tokens": parsed["usage"].input_tokens,
                "output_tokens": parsed["usage"].output_tokens,
            },
        )

        return parsed

    # -- request building --

    @staticmethod
    def _build_kwargs(state: PipelineState) -> Dict[str, Any]:
        kwargs: Dict[str, Any] = {
            "model": state.model,
            "messages": list(state.messages),
            "max_tokens": state.max_tokens,
        }

        if state.system:
            kwargs["system"] = state.system

        if state.temperature is not None:
            kwargs["temperature"] = state.temperature
        if state.tools:
            kwargs["tools"] = state.tools
        if state.tool_choice:
            kwargs["tool_choice"] = state.tool_choice
        if state.stop_sequences:
            kwargs["stop_sequences"] = state.stop_sequences
        if state.thinking_enabled:
            kwargs["thinking"] = {
                "type": "enabled",
                "budget_tokens": state.thinking_budget_tokens,
            }

        return kwargs

    # -- streaming --

    async def _call_streaming(
        self, kwargs: Dict[str, Any], state: PipelineState
    ) -> Dict[str, Any]:
        parsed: Optional[Dict[str, Any]] = None

        async for chunk in self._provider.create_message_stream(kwargs):
            chunk_type = chunk.get("type")
            if chunk_type == "message_complete":
                parsed = _parse_raw_response(chunk["response"])
            elif chunk_type == "text_delta" and chunk.get("text"):
                state.add_event("text.delta", {"text": chunk["text"]})

        if parsed is None:
            raise APIError(
                "Stream ended without message_complete",
                category=ErrorCategory.UNKNOWN,
            )
        return parsed

    # -- non-streaming with retry --

    async def _call_with_retry(
        self, kwargs: Dict[str, Any], state: PipelineState
    ) -> Dict[str, Any]:
        last_error: Optional[Exception] = None

        for attempt in range(self._max_retries + 1):
            try:
                raw = await self._provider.create_message(kwargs)
                return _parse_raw_response(raw)
            except APIError as e:
                last_error = e
                if not e.category.is_recoverable or attempt >= self._max_retries:
                    raise
                delay = min(2 ** attempt, 8)
                state.add_event(
                    "api.retry",
                    {"attempt": attempt + 1, "category": e.category.value, "delay": delay},
                )
                await asyncio.sleep(delay)
            except Exception as e:
                raise APIError(str(e), category=ErrorCategory.UNKNOWN, cause=e) from e

        raise last_error or APIError("Max retries exceeded", category=ErrorCategory.UNKNOWN)

    # -- assistant content --

    @staticmethod
    def _build_assistant_content(parsed: Dict[str, Any]) -> Any:
        blocks = parsed.get("content_blocks", [])
        if not blocks:
            return parsed.get("text", "")
        if len(blocks) == 1 and blocks[0].get("type") == "text":
            return blocks[0].get("text", "")
        return blocks

    def list_strategies(self) -> List[StrategyInfo]:
        si = StrategyInfo(
            slot_name="provider",
            current_impl=type(self._provider).__name__,
        )
        si.available_impls = ["AnthropicProvider"]
        return [si]
