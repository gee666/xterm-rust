# Parser type mapping

This is a symbol-level map for `source_repo/src/common/parser/Types.ts`. It does not mark the whole file covered by one criterion or claim that all browser/runtime checks have passed.

## Parameters

`IParams`, `ParamsArray` and `IParamsConstructor` map to `parser::params::{Params, ParamValue}` and the checked constructors/mutation methods described in `docs/params.md`. Criteria 110-113 own those representations, numeric boundaries, borrowed arrays and assertion evidence. They are not inferred from the handler mappings below. Raw custom PARAM numeric coercion is additional work in task 178; a strict decimal-digit API alone is not proof of that source path.

## Identifiers and callback types, criterion 33

| Source symbol | Rust contract |
| --- | --- |
| `IFunctionIdentifier` | `escape::FunctionIdentifier` for native borrowed strings. `packed` retains validation order, UTF-16 lengths and exact source error messages. Dynamic JavaScript reads, repeated getters and APC setter effects belong to binding task 214. |
| `PrintHandlerType`, `PrintFallbackHandlerType` | A borrowed `&[u32]` replaces the source data/start/end triple. Set/clear print methods retain the default no-handler behavior. Checked setters propagate backend errors. |
| `ExecuteHandlerType`, `ExecuteFallbackHandlerType` | Execute receives `Option<u32>` to preserve the source's fast-path code argument versus omitted table-path argument. Fallback always receives the code. Checked variants stop before join-state clearing and subsequent bytes. |
| `CsiHandlerType` | `escape::CommandHandler`, a shared callback identity accepting borrowed Params and returning `HandlerResult`. The public ParserApi separately supplies a fresh owned parameter-array snapshot. |
| `EscHandlerType` | The same registration/dispatch machinery, with Params unused by ESC wrappers. Public ESC callbacks receive no arguments. |
| `CsiFallbackHandlerType`, `EscFallbackHandlerType` | Packed identifiers retain their 32-bit bits. CSI additionally borrows Params. Both have infallible convenience setters and checked Result setters, not delayed-error side channels. |
| `IParsingState` | Borrowed `escape::ParsingState` exposes position, code, state, collect and Params. Returning true replaces the source abort flag. The source does not implement injection of other modified state fields. Checked error callbacks preserve original failures. |
| `IHandlerCollection<T>` | Shared `handler::registry::HandlerRegistry` and live handler-list identities. Weak registration handles and disposal tokens prevent callback-own-token cycles. Explicit disposal remains observable; dropping a token does not unregister. |
| `IDcsHandler` | `dcs::DcsHandlerTrait` hook/put/unhook, with borrowed Params and UTF-32 slices. Hook/put errors propagate immediately. Completed-payload adapters separately clone Params as source requires. |
| `IOscHandler`, `IApcHandler` | `osc::OscHandlerTrait` and its APC alias. Shared dispatch does not merge their distinct identifier parsing/start rules. Raw start/put errors propagate immediately; end returns the shared handler result. |
| `DcsFallbackHandlerType` | `dcs::DcsAction` distinguishes Hook/Put/Unhook instead of action strings and an untyped payload. String payloads use UTF-16 units. |
| `OscFallbackHandlerType`, `ApcFallbackHandlerType` | `OscAction`/APC action alias distinguish Start/Put/End. OSC identifiers retain JavaScript Number semantics; APC uses the packed identifier bits. |
| `IEscapeSequenceParser` | `escape::EscapeSequenceParser` owns actual transitions, Params and all three streaming parsers. Registration, clear, fallback, reset and explicit disposal methods are working core methods, not JS terminal delegation. |
| `ISubParser<T,U>`, `IOscParser`, `IDcsParser`, `IApcParser` | Concrete `OscParser`, `DcsParser` and `ApcParser` expose typed register/clear/fallback/put/reset/dispose and protocol-specific start/hook/end methods. Result errors preserve partial state rather than returning success after a callback failure. |

`CommandRegistrar::register_packed` and `clear_packed` provide a binding route for an identifier already computed with source getter order. They must not read a second object snapshot or reclassify CSI t security from the final packed byte. The binding supplies its earlier guard decision explicitly. Task 214 verifies real property callbacks and actual dispatch, not only the integer key.

## Continuations, criterion 34

| Source symbol | Rust representation |
| --- | --- |
| `ParserStackType.NONE` | `ParseStackState::None`. |
| `ParserStackType.FAIL` | Sticky `ParseStackState::Failed`, cleared only by reset. |
| `ParserStackType.RESET` | `ParseStackState::Reset` with the saved chunk position; the next parse resumes at the following codepoint in the initial state. |
| `ParserStackType.CSI/ESC/OSC/DCS/APC` | `ParseStackState::Paused` plus the corresponding `HandlerKind`. The enum is structured state, not a promise of identical native discriminants. |
| `IParserStackState` | A private pause frame retains live handlers, next handler position, transition and chunk position. `pause_info()` exposes read-only diagnostic fields for exact original pause assertions. |
| `ResumableHandlersType` | A retained shared handler-list identity, using the same reverse dispatch rules for CSI and ESC. |
| `ISubParserStackState` | A private optional subparser pause frame records remaining position and cleanup/fallthrough state. It preserves subsequent handler order while the caller owns the returned future. |

`HandlerResult::Ready`, `Pending`, `PendingFallible` and `Failed` distinguish sync results, ordinary futures, rejected futures and immediate failures. Resolve a pending result and resume only on success, using the identical chunk. A rejection is not a false handler result. Reset cleanup futures must be driven at the documented executor turns. See `docs/escape-parser.md` and the OSC/DCS/APC documents for ordering and retained failures.

The exact original outer-parser evidence is 161 source cases: 147 synchronous cases mapped by task 90 and 14 async cases mapped by task 26. `results/escape-parser/async-mapping.json` accounts for the latter. Read current command receipts before treating a mapping as an executed pass. Normal state/action data belongs to criteria 114-116. Remaining raw custom table/state/number behavior belongs to task 178, not an undocumented type restriction.
