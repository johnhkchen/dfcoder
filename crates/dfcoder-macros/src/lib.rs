use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr, Token};

mod scenario;
mod agent;
mod events;
mod baml;
mod mcp;

use scenario::ScenarioInput;
use agent::AgentInput;
use events::EventsInput;
use baml::BamlSchemaInput;
use mcp::McpResourcesInput;

/// Natural language test scenario DSL
/// 
/// # Example
/// ```
/// scenario! {
///     "Agent requests help when stuck"
///     given: agent working on complex task for 2 minutes,
///     when: no progress detected,
///     then: supervisor sees dialogue with context;
/// }
/// ```
#[proc_macro]
pub fn scenario(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ScenarioInput);
    scenario::expand(input).unwrap_or_else(|e| e.to_compile_error()).into()
}

/// Agent behavior definition DSL
///
/// # Example
/// ```
/// agent! {
///     RustExpert responds to "type error" with careful_analysis,
///     when idle: monitors for rust files,
///     during supervision: provides context within 10 lines;
/// }
/// ```
#[proc_macro]
pub fn agent(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AgentInput);
    agent::expand(input).unwrap_or_else(|e| e.to_compile_error()).into()
}

/// Event flow definition DSL
///
/// # Example
/// ```
/// events! {
///     from Agent to Supervisor: RequestGuidance { context, options },
///     from Supervisor to Agent: ProvideGuidance { choice };
/// }
/// ```
#[proc_macro]
pub fn events(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as EventsInput);
    events::expand(input).unwrap_or_else(|e| e.to_compile_error()).into()
}

/// BAML schema definition DSL
///
/// # Example
/// ```
/// baml_schema! {
///     activities categorize as {
///         CodeGeneration { creating, refactoring, testing },
///         ProblemSolving { debugging, researching, analyzing };
///     }
/// }
/// ```
#[proc_macro]
pub fn baml_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as BamlSchemaInput);
    baml::expand(input).unwrap_or_else(|e| e.to_compile_error()).into()
}

/// MCP resource definition DSL
///
/// # Example
/// ```
/// mcp_resources! {
///     resource agents {
///         list: active_agents with status,
///         read: agent_state(id: AgentId),
///         write: send_command(id: AgentId, cmd: Command);
///     }
/// }
/// ```
#[proc_macro]
pub fn mcp_resources(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as McpResourcesInput);
    mcp::expand(input).unwrap_or_else(|e| e.to_compile_error()).into()
}