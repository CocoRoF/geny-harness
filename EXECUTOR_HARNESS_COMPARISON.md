# geny-executor vs geny-harness 심층 비교 분석

> **목적**: Rust(geny-harness)가 Python(geny-executor)의 모든 실제 실행 로직을 완전히 대체할 수 있는지 검증
> **기준 버전**: geny-harness 0.5.3 / geny-executor 최신
> **작성일**: 2026-04-12

---

## 1. PipelineState 필드 완전 비교

| 필드 | Python (executor) | Rust (harness) | PyO3 getter | PyO3 setter | sync_state_to_python | 상태 |
|------|-------------------|----------------|-------------|-------------|----------------------|------|
| **Identity** |
| session_id | `str` | `String` | O | O | - (extract_state) | OK |
| pipeline_id | `str` | `String` | O | O | - | OK |
| **Messages** |
| system | `Union[str, List[Dict]]` | `Value` | O | O | O | OK |
| messages | `List[Dict]` | `Vec<Value>` | O | O | O | OK |
| **Execution** |
| iteration | `int` | `u32` | O | O | O | OK |
| max_iterations | `int` (50) | `u32` (50) | O | O | - (extract_state) | OK |
| current_stage | `str` | `String` | O | O | - | OK |
| stage_history | `List[str]` | `Vec<String>` | O | **X** | - | **MINOR** |
| **Model** |
| model | `str` | `String` | O | O | O | OK |
| max_tokens | `int` (8192) | `u32` (8192) | O | O | - | OK |
| temperature | `float` (0.0) | `f64` (0.0) | O | O | - | OK |
| tools | `List[Dict]` | `Vec<Value>` | O | O | - | OK |
| tool_choice | `Optional[Dict]` | `Option<Value>` | **X** | **X** | - | **GAP** |
| stop_sequences | `Optional[List[str]]` | `Option<Vec<String>>` | **X** | **X** | - | **GAP** |
| **Extended Thinking** |
| thinking_enabled | `bool` | `bool` | O | O | - | OK |
| thinking_budget_tokens | `int` (10000) | `u32` (10000) | O | O | - | OK |
| thinking_history | `List[Dict]` | `Vec<Value>` | **X** | **X** | - | **GAP** |
| **Token & Cost** |
| token_usage | `TokenUsage` | `TokenUsage` | O | O | O | OK |
| turn_token_usage | `List[TokenUsage]` | `Vec<TokenUsage>` | **X** | **X** | - | **GAP** |
| total_cost_usd | `float` | `f64` | O | O | O | OK |
| cost_budget_usd | `Optional[float]` | `Option<f64>` | O | O | - | OK |
| **Cache** |
| cache_metrics | `CacheMetrics` | `CacheMetrics` | O | O | O | OK |
| **Context** |
| memory_refs | `List[Dict]` | `Vec<Value>` | **X** | **X** | - | **GAP** |
| context_window_budget | `int` (200000) | `u32` (200000) | O | O | - | OK |
| **Loop Control** |
| loop_decision | `str` | `String` | O | O | O | OK |
| completion_signal | `Optional[str]` | `Option<String>` | O | O | O | OK |
| completion_detail | `Optional[str]` | `Option<String>` | O | O | O | OK |
| **Tool Execution** |
| pending_tool_calls | `List[Dict]` | `Vec<Value>` | O | O | O | OK |
| tool_results | `List[Dict]` | `Vec<Value>` | O | O | O | OK |
| **Agent** |
| delegate_requests | `List[Dict]` | `Vec<Value>` | **X** | **X** | - | **GAP** |
| agent_results | `List[Dict]` | `Vec<Value>` | **X** | **X** | - | **GAP** |
| **Evaluation** |
| evaluation_score | `Optional[float]` | `Option<f64>` | **X** | **X** | - | **GAP** |
| evaluation_feedback | `Optional[str]` | `Option<String>` | **X** | **X** | - | **GAP** |
| **Output** |
| final_text | `str` | `String` | O | O | O | OK |
| final_output | `Optional[Any]` | `Option<Value>` | O | O | O | OK |
| **Debug** |
| last_api_response | `Optional[Any]` | `Option<Value>` | **X** | **X** | - | **GAP** |
| **Metadata** |
| created_at | `datetime` | `DateTime<Utc>` | **X** | **X** | - | **MINOR** |
| updated_at | `datetime` | `DateTime<Utc>` | **X** | **X** | - | **MINOR** |
| metadata | `Dict[str, Any]` | `Map<String, Value>` | **X** | **X** | - | **GAP** |
| events | `List[Dict]` | `Vec<Value>` | O | **X** (read-only) | - (skip) | OK |
| _event_listener | internal | internal | - | - | - | OK |

### 요약

- **완전 일치**: 24개 필드
- **PyO3 미노출 (GAP)**: 12개 필드 — Rust 내부에는 존재하지만 Python에서 접근 불가
- **경미한 차이 (MINOR)**: 3개 — stage_history setter 없음, created_at/updated_at 미노출

---

## 2. Stage 실행 로직 비교

### Stage 1: Input (ingress)

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 입력 검증 | InputValidator (Default, Passthrough, Strict, Schema) | DefaultValidator | **PARTIAL** |
| 입력 정규화 | InputNormalizer (Default, Multimodal) | DefaultNormalizer | **PARTIAL** |
| 메시지 추가 | `state.messages.append({"role":"user","content":...})` | `state.add_message("user", content)` | OK |
| 멀티모달 지원 | MultimodalNormalizer (images, files) | 텍스트 전용 | **GAP** |
| 이벤트 | "input.normalized" | "input.normalized" | OK |

**결론**: 기본 텍스트 입력은 동일. **멀티모달(이미지, 파일)** 입력은 Rust에서 미지원.

---

### Stage 2: Context (ingress)

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 컨텍스트 전략 | Simple, Hybrid, ProgressiveDisclosure | SimpleLoad | **PARTIAL** |
| 메모리 검색 | NullRetriever, StaticRetriever | NullRetriever | OK |
| 히스토리 압축 | Truncate, Summary, SlidingWindow | Truncate | **PARTIAL** |
| memory_refs 저장 | `state.memory_refs.append(...)` | `state.memory_refs.push(...)` | OK |
| 컨텍스트 버짓 체크 | 80% 임계값에서 압축 실행 | 동일 | OK |

**결론**: 기본 컨텍스트 관리는 동등. 고급 전략(Hybrid, SlidingWindow)은 Rust 미구현.

---

### Stage 3: System (ingress)

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 프롬프트 빌더 | Static, Composable | Static | **PARTIAL** |
| 도구 등록 | `state.tools = registry.to_api_format()` | 동일 | OK |
| 캐시 마커 지원 | 블록 형태 시스템 프롬프트 | 동일 | OK |

**결론**: StaticPromptBuilder만 사용하는 경우 동등. ComposablePromptBuilder 미구현.

---

### Stage 4: Guard (pre_flight)

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 가드 체인 | TokenBudget, CostBudget, Iteration, Permission | CostBudget, Iteration, ContextLimit | **PARTIAL** |
| 거부 에러 | `GuardRejectError` | `StageError` | **DIFF** |
| 경고 동작 | action="warn" → 경고 후 계속 | 동일 | OK |

**결론**: 핵심 가드(비용, 반복)는 동일. TokenBudgetGuard, PermissionGuard 미구현. 에러 타입 차이.

---

### Stage 5: Cache (pre_flight)

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 전략 | NoCache, SystemCache, AggressiveCache | NoCache, SystemCache, AggressiveCache | **OK** |
| 캐시 마커 삽입 | system 블록에 `cache_control` 추가 | 동일 | OK |
| 바이패스 | NoCacheStrategy면 bypass | 동일 | OK |

**결론**: **완전 일치**.

---

### Stage 6: API (execution) ⭐ 핵심 스테이지

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| API 프로바이더 | AnthropicProvider (anthropic SDK) | AnthropicProvider (reqwest) | OK |
| 스트리밍 | `create_message_stream()` → text.delta 이벤트 | 동일 구조 | OK |
| 재시도 | ExponentialBackoff, NoRetry, RateLimitAware | ExponentialBackoff | **PARTIAL** |
| 요청 빌드 | model, max_tokens, temperature, system, messages, tools, tool_choice, thinking | 동일 | OK |
| stop_sequences | 지원 | **state에서 읽지만 PyO3 미노출** | **GAP** |
| tool_choice | 지원 | **state에서 읽지만 PyO3 미노출** | **GAP** |
| last_api_response 저장 | `state.last_api_response = response` | 동일 | OK |
| 어시스턴트 메시지 추가 | `state.messages.append({"role":"assistant",...})` | 동일 | OK |
| 텍스트 스트리밍 | text.delta 이벤트 실시간 발행 | 동일 | OK |

**결론**: 핵심 API 호출 로직 동등. `stop_sequences`, `tool_choice`는 Rust 내부에 있지만 **PyO3에서 set 불가**.

---

### Stage 7: Token (execution)

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 토큰 추적 | DefaultTracker, DetailedTracker | DefaultTracker | **PARTIAL** |
| 비용 계산 | AnthropicPricing, CustomPricing | AnthropicPricing | **PARTIAL** |
| token_usage 누적 | `state.token_usage += turn_usage` | 동일 | OK |
| turn_token_usage | `state.turn_token_usage.append(usage)` | 동일 (Rust 내부) | OK |
| cache_metrics 갱신 | writes, reads 카운트 | 동일 | OK |
| total_cost_usd 누적 | `state.accumulate_cost(cost)` | 동일 | OK |

**결론**: 기본 추적/비용 계산 동등. DetailedTracker, CustomPricing 미구현.

---

### Stage 8: Think (execution)

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| thinking 블록 분리 | 응답에서 thinking 블록 추출 | 동일 | OK |
| 프로세서 | ExtractAndStore, Passthrough, Filter | Passthrough | **PARTIAL** |
| thinking_history 저장 | `state.thinking_history.append(...)` | 동일 | OK |
| 바이패스 | `thinking_enabled == False` | 동일 | OK |

**결론**: Passthrough만 사용하는 경우 동등.

---

### Stage 9: Parse (execution) ⭐ 핵심 스테이지

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 파서 | Default, StructuredOutput | Default | **PARTIAL** |
| 시그널 감지 | Regex, Structured, Hybrid | Regex, Structured, Hybrid | **OK** |
| final_text 설정 | `state.final_text = parsed.text` | 동일 | OK |
| pending_tool_calls | tool_use 블록 → `state.pending_tool_calls` | 동일 | OK |
| completion_signal | 감지 결과 → `state.completion_signal` | 동일 | OK |
| thinking_history | thinking 텍스트 추출 → 저장 | 동일 | OK |

**결론**: 핵심 파싱 로직 동등. StructuredOutputParser 미구현.

---

### Stage 10: Tool (execution) ⭐ 핵심 스테이지

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 실행기 | Sequential, Parallel | Sequential | **PARTIAL** |
| 라우터 | RegistryRouter | RegistryRouter | OK |
| 결과 처리 | tool_results → user 메시지 추가 | 동일 | OK |
| loop_decision | "continue" 설정 | 동일 | OK |
| pending_tool_calls 클리어 | 실행 후 초기화 | 동일 | OK |
| 바이패스 | pending_tool_calls 비어있으면 | 동일 | OK |

**결론**: Sequential 실행은 동등. ParallelExecutor 미구현.

---

### Stage 11: Agent (execution)

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 오케스트레이터 | SingleAgent, Delegate, Evaluator | SingleAgent | **PARTIAL** |
| 서브 파이프라인 위임 | DelegateOrchestrator 지원 | 미지원 | **GAP** |
| delegate_requests PyO3 | N/A | **미노출** | **GAP** |

**결론**: SingleAgent(패스스루)만 동등. 멀티에이전트 위임은 미구현.

---

### Stage 12: Evaluate (decision)

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 평가 전략 | SignalBased, CriteriaBased, Agent | SignalBased | **PARTIAL** |
| 스코어러 | NoScorer, Weighted | NoScorer | **PARTIAL** |
| evaluation_score PyO3 | N/A | **미노출** | **GAP** |
| loop_decision 매핑 | complete→complete, continue→continue, etc. | 동일 | OK |

**결론**: SignalBased 평가는 동등. 고급 평가(Criteria, Agent) 미구현.

---

### Stage 13: Loop (decision)

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 컨트롤러 | Standard, SingleTurn, BudgetAware | Standard | **PARTIAL** |
| 표준 로직 | tool_results→continue, signal→complete, etc. | 동일 | OK |
| tool_results 클리어 | 매 루프 끝에 초기화 | 동일 | OK |
| MAX_ITERATIONS 강제 종료 | pipeline.rs에서 처리 | 동일 | OK |

**결론**: StandardLoopController 동작 동등. BudgetAwareLoopController 미구현.

---

### Stage 14: Emit (egress)

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 이미터 | Text, Callback, VTuber, TTS | EmitterChain (빈 체인) | **GAP** |
| VTuber 통합 | 전용 이미터 | 미구현 | **GAP** |

**결론**: 이미터 프레임워크는 있으나 **구체적 이미터 구현 없음**.

---

### Stage 15: Memory (egress)

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 전략 | AppendOnly, NoMemory, Reflective | AppendOnly | **PARTIAL** |
| 영속화 | DB, File, Vector 백엔드 | 없음 | **GAP** |
| stateless 바이패스 | metadata["stateless"] 체크 | 동일 | OK |

**결론**: 메모리 전략 프레임워크 동등. **실제 영속화 백엔드 없음**.

---

### Stage 16: Yield (egress)

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| 포매터 | Default, Structured, Streaming | Default | **PARTIAL** |
| final_output 반환 | final_output ?? final_text | 동일 | OK |

**결론**: 기본 동작 동등.

---

## 3. PyO3 바인딩 갭 분석

Python 코드에서 Rust PipelineState의 특정 필드에 접근할 수 없는 경우, 해당 필드를 사용하는 Python 코드(웹 UI 등)가 정상 작동하지 않습니다.

### 3.1 누락된 getter/setter

| 필드 | 중요도 | 영향 | 비고 |
|------|--------|------|------|
| `tool_choice` | **HIGH** | API 스테이지에서 tool_choice 강제 불가 | `{"type": "auto"}` 등 설정 필요 |
| `stop_sequences` | **MED** | 커스텀 stop 시퀀스 설정 불가 | API 호출 시 누락 |
| `thinking_history` | **MED** | thinking 기록 조회 불가 | 디버깅/UI 표시용 |
| `turn_token_usage` | **LOW** | 턴별 토큰 사용량 조회 불가 | 상세 모니터링용 |
| `memory_refs` | **LOW** | 메모리 참조 조회 불가 | 컨텍스트 디버깅용 |
| `last_api_response` | **LOW** | 원본 API 응답 조회 불가 | 디버깅용 |
| `metadata` | **MED** | 임의 메타데이터 저장/조회 불가 | 스테이지 간 데이터 전달 |
| `delegate_requests` | **LOW** | 멀티에이전트 미사용 시 무관 | 미래 기능 |
| `agent_results` | **LOW** | 멀티에이전트 미사용 시 무관 | 미래 기능 |
| `evaluation_score` | **LOW** | 평가 점수 조회 불가 | UI 표시용 |
| `evaluation_feedback` | **LOW** | 평가 피드백 조회 불가 | UI 표시용 |
| `stage_history` (setter) | **LOW** | 외부에서 히스토리 수정 불가 | 거의 불필요 |

### 3.2 sync_state_to_python 누락 필드

`run` 또는 `run_stream` 실행 후 Python 상태로 동기화되지 않는 필드:

| 필드 | 동기화 여부 | 영향 |
|------|-------------|------|
| messages | **O** | 멀티턴 대화 유지 |
| system | **O** | 시스템 프롬프트 유지 |
| iteration | **O** | 반복 카운터 |
| token_usage | **O** | 누적 토큰 |
| total_cost_usd | **O** | 누적 비용 |
| cache_metrics | **O** | 캐시 통계 |
| loop_decision | **O** | 루프 결정 |
| final_text | **O** | 최종 텍스트 |
| final_output | **O** | 구조화 출력 |
| pending_tool_calls | **O** | 도구 호출 |
| tool_results | **O** | 도구 결과 |
| **thinking_history** | **X** | thinking 블록 누실 |
| **turn_token_usage** | **X** | 턴별 토큰 누실 |
| **metadata** | **X** | 메타데이터 누실 |
| **evaluation_score** | **X** | 평가 점수 누실 |

---

## 4. extract_state (Python→Rust) 갭 분석

`PyPipeline.extract_state()`가 Python 상태에서 Rust로 복사하는 필드:

| 필드 | 복사 여부 | 문제 |
|------|-----------|------|
| session_id | **O** | |
| model | **O** | |
| max_iterations | **O** | |
| messages | **O** | |
| system | **O** | |
| **iteration** | **X** | 멀티턴 시 반복 카운터 초기화됨 |
| **total_cost_usd** | **X** | 멀티턴 시 누적 비용 초기화됨 |
| **token_usage** | **X** | 멀티턴 시 누적 토큰 초기화됨 |
| **tools** | **X** | 도구 정보 유실 |
| **thinking_enabled** | **X** | thinking 설정 유실 |
| **max_tokens** | **X** | 토큰 제한 유실 |
| **temperature** | **X** | 온도 설정 유실 |
| **cost_budget_usd** | **X** | 비용 예산 유실 |
| **tool_choice** | **X** | 도구 선택 설정 유실 |
| **stop_sequences** | **X** | stop 시퀀스 유실 |
| **context_window_budget** | **X** | 컨텍스트 버짓 유실 |
| **cache_metrics** | **X** | 캐시 통계 유실 |
| **metadata** | **X** | 메타데이터 유실 |

> **심각도: HIGH** — 현재 `extract_state`는 5개 필드만 복사. 멀티턴 세션에서 iteration, cost, token_usage 등이 매 턴마다 0으로 초기화되어 **비용 추적 불가**, **반복 제한 무효화** 문제 발생.

---

## 5. Preset 비교

| 프리셋 | Python 스테이지 | Rust 스테이지 | 일치 |
|--------|----------------|---------------|------|
| **minimal** | 1→6→9→16 | 1→6→9→16 | **OK** |
| **chat** | 1,2,3,4,5,6,7,9,10,13,15,16 | 1,2,3,4,5(system),6,7,9,10,13,15,16 | **OK** |
| **agent** | 모든 16 스테이지 | 모든 16 스테이지 | **OK** |
| **evaluator** | 1,3,6,9,12,16 | 1,3,6,9,12,16 | **OK** |
| **geny_vtuber** | 모든 16 + VTuber 이미터 | 모든 16 (이미터 없음) | **PARTIAL** |

---

## 6. 세션 관리 비교

| 항목 | Python (executor) | Rust (harness) | 일치 |
|------|-------------------|----------------|------|
| SessionManager | dict 기반 저장 | 동일 구조 | OK |
| Session.run() | state 참조 전달 (in-place 수정) | state 복사 → sync_state_to_python | **DIFF** |
| Session.run_stream() | state 참조 전달 | state 복사 → sync_state | **DIFF** |
| FreshnessPolicy | 동일 기본값 | 동일 기본값 | OK |
| FreshnessStatus | 5단계 (Fresh~StaleReset) | 5단계 (동일) | OK |
| reset_state() | 새 PipelineState 생성 | 동일 | OK |

> **핵심 차이**: Python은 `state`를 **참조로 전달**하여 파이프라인이 직접 수정. Rust는 **복사 후 실행 → 결과를 다시 동기화**. 이 구조적 차이로 인해 `extract_state`와 `sync_state_to_python`의 완전성이 중요.

---

## 7. 우선순위별 수정 필요 사항

### P0: 즉시 수정 (실행 정확성에 영향)

| # | 항목 | 설명 | 위치 |
|---|------|------|------|
| 1 | **extract_state 확장** | iteration, total_cost_usd, token_usage, tools, thinking_enabled, max_tokens, temperature, cost_budget_usd, context_window_budget, cache_metrics 복사 추가 | `lib.rs` extract_state() |
| 2 | **sync_state 확장** | thinking_history, turn_token_usage, metadata 동기화 추가 | `lib.rs` sync_state_to_python() |
| 3 | **tool_choice PyO3 노출** | getter/setter 추가 | `lib.rs` PyPipelineState |
| 4 | **stop_sequences PyO3 노출** | getter/setter 추가 | `lib.rs` PyPipelineState |

### P1: 중기 수정 (기능 완전성)

| # | 항목 | 설명 |
|---|------|------|
| 5 | thinking_history PyO3 노출 | getter/setter 추가 |
| 6 | turn_token_usage PyO3 노출 | getter/setter 추가 |
| 7 | metadata PyO3 노출 | getter/setter 추가 |
| 8 | last_api_response PyO3 노출 | getter/setter 추가 |
| 9 | memory_refs PyO3 노출 | getter/setter 추가 |
| 10 | ParallelExecutor (Stage 10) | 도구 병렬 실행 |

### P2: 장기 (고급 기능)

| # | 항목 | 설명 |
|---|------|------|
| 11 | StructuredOutputParser (Stage 9) | 구조화 출력 파싱 |
| 12 | ComposablePromptBuilder (Stage 3) | 동적 프롬프트 조합 |
| 13 | DelegateOrchestrator (Stage 11) | 멀티에이전트 위임 |
| 14 | CriteriaBased/AgentEvaluation (Stage 12) | 고급 평가 |
| 15 | BudgetAwareLoopController (Stage 13) | 예산 인식 루프 |
| 16 | VTuber/TTS 이미터 (Stage 14) | 실제 이미터 구현 |
| 17 | Memory 영속화 백엔드 (Stage 15) | DB/파일 저장 |
| 18 | MultimodalNormalizer (Stage 1) | 이미지/파일 입력 |

---

## 8. 최종 결론

### 현재 상태 요약

```
전체 기능 커버리지: ~70%

핵심 파이프라인 실행:     ████████████████████ 95%  (실행 흐름 동일)
State 필드 일치:          ████████████████░░░░ 80%  (12개 필드 PyO3 미노출)
extract_state 완전성:     ██████░░░░░░░░░░░░░░ 30%  (5/20 필드만 복사) ← 가장 심각
sync_state 완전성:        ██████████████░░░░░░ 70%  (14/20 필드 동기화)
Stage 전략 다양성:        ██████████░░░░░░░░░░ 50%  (대부분 default만 구현)
세션 영속성:              ██████████████░░░░░░ 65%  (extract_state 부족으로 감점)
```

### 핵심 문제

1. **`extract_state`가 5개 필드만 복사** — 멀티턴에서 iteration, cost, token이 매번 초기화됨. `sync_state_to_python`으로 돌려주지만 다음 턴에서 다시 `extract_state`로 읽을 때 누락됨.

2. **tool_choice, stop_sequences PyO3 미노출** — Python에서 설정 불가, API 호출 시 누락 가능.

3. **Stage 전략이 default만 구현** — 실 서비스에서 사용하는 고급 전략(Aggressive Cache, BudgetAware Loop 등)은 이미 Rust에 있으므로 문제 없으나, 일부(Parallel Tool, Structured Output)는 미구현.

### 즉시 조치 권장

**`extract_state` 확장이 가장 시급합니다.** 현재 멀티턴 세션에서:
- 2턴째부터 `iteration=0`으로 리셋 → max_iterations 가드 무효화
- 2턴째부터 `total_cost_usd=0` → 비용 추적 리셋
- 2턴째부터 `token_usage` 초기화 → 누적 토큰 추적 실패
- `tools`가 복사 안 됨 → config에서 설정한 도구가 2턴째 사라질 수 있음
