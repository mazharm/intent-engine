//! Code generation from Template intents

use std::collections::HashMap;

use crate::parser::IntentStore;

/// Process all Template intents and generate output files
pub fn generate_from_templates(
    store: &IntentStore,
    context: &TemplateContext,
) -> Vec<TemplateOutput> {
    let mut outputs = Vec::new();

    let templates: Vec<_> = store.templates().into_iter().collect();

    for doc in templates {
        if let Ok(spec) = doc.as_template_spec() {
            let content = render_template(&spec.template, context, &spec.partials);
            let output_path = interpolate_path(&spec.output_file, context);
            outputs.push(TemplateOutput {
                path: output_path,
                content,
                source_intent: doc.id.to_string(),
            });
        }
    }

    outputs
}

/// Output from template rendering
#[derive(Debug)]
pub struct TemplateOutput {
    pub path: String,
    pub content: String,
    pub source_intent: String,
}

/// Context data for template rendering
#[derive(Debug, Default)]
pub struct TemplateContext {
    pub version: String,
    pub types: Vec<TypeInfo>,
    pub endpoints: Vec<EndpointInfo>,
    pub workflows: Vec<WorkflowInfo>,
    pub enums: Vec<EnumInfo>,
    pub functions: Vec<FunctionInfo>,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub rust_type: String,
    pub required: bool,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct EndpointInfo {
    pub name: String,
    pub method: String,
    pub path: String,
    pub input: String,
    pub output: String,
    pub workflow: String,
}

#[derive(Debug, Clone)]
pub struct WorkflowInfo {
    pub name: String,
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub params: Vec<String>,
    pub returns: String,
}

/// Render a template with the given context
fn render_template(
    lines: &[String],
    context: &TemplateContext,
    partials: &HashMap<String, Vec<String>>,
) -> String {
    let mut output = String::new();
    let mut iter = lines.iter().peekable();

    while let Some(line) = iter.next() {
        let rendered = render_line(line, &mut iter, context, partials);
        output.push_str(&rendered);
        output.push('\n');
    }

    output
}

fn render_line(
    line: &str,
    iter: &mut std::iter::Peekable<std::slice::Iter<String>>,
    context: &TemplateContext,
    partials: &HashMap<String, Vec<String>>,
) -> String {
    // Check for control structures
    if line.trim().starts_with("{{#each") {
        return handle_each(line, iter, context, partials);
    }
    if line.trim().starts_with("{{#if") {
        return handle_if(line, iter, context, partials);
    }
    if line.trim().starts_with("{{>") {
        return handle_partial(line, context, partials);
    }

    // Simple variable interpolation
    interpolate_variables(line, context)
}

fn handle_each(
    line: &str,
    iter: &mut std::iter::Peekable<std::slice::Iter<String>>,
    context: &TemplateContext,
    partials: &HashMap<String, Vec<String>>,
) -> String {
    // Extract the collection name
    let start = line.find("{{#each").unwrap() + 7;
    let end = line.find("}}").unwrap();
    let collection_name = line[start..end].trim();

    // Collect the body until {{/each}}
    let mut body = Vec::new();
    while let Some(inner_line) = iter.next() {
        if inner_line.trim().starts_with("{{/each}}") {
            break;
        }
        body.push(inner_line.clone());
    }

    // Iterate over the collection
    let mut output = String::new();

    match collection_name {
        "types" => {
            for type_info in &context.types {
                let type_context = create_type_context(type_info, context);
                for body_line in &body {
                    output.push_str(&render_line(
                        body_line,
                        &mut body.iter().peekable(),
                        &type_context,
                        partials,
                    ));
                    output.push('\n');
                }
            }
        }
        "fields" => {
            // This would be handled in a nested context
        }
        "endpoints" => {
            for endpoint_info in &context.endpoints {
                let endpoint_context = create_endpoint_context(endpoint_info, context);
                for body_line in &body {
                    output.push_str(&render_line(
                        body_line,
                        &mut body.iter().peekable(),
                        &endpoint_context,
                        partials,
                    ));
                    output.push('\n');
                }
            }
        }
        _ => {}
    }

    output
}

fn handle_if(
    line: &str,
    iter: &mut std::iter::Peekable<std::slice::Iter<String>>,
    context: &TemplateContext,
    partials: &HashMap<String, Vec<String>>,
) -> String {
    // Extract condition
    let start = line.find("{{#if").unwrap() + 5;
    let end = line.find("}}").unwrap();
    let condition = line[start..end].trim();

    // Collect body and else
    let mut then_body = Vec::new();
    let mut else_body = Vec::new();
    let mut in_else = false;

    while let Some(inner_line) = iter.next() {
        if inner_line.trim().starts_with("{{/if}}") {
            break;
        }
        if inner_line.trim().starts_with("{{else}}") {
            in_else = true;
            continue;
        }
        if in_else {
            else_body.push(inner_line.clone());
        } else {
            then_body.push(inner_line.clone());
        }
    }

    // Evaluate condition
    let condition_value = evaluate_condition(condition, context);

    let body = if condition_value { &then_body } else { &else_body };
    let mut output = String::new();
    for body_line in body {
        output.push_str(&render_line(
            body_line,
            &mut body.iter().peekable(),
            context,
            partials,
        ));
        output.push('\n');
    }

    output
}

fn handle_partial(
    line: &str,
    context: &TemplateContext,
    partials: &HashMap<String, Vec<String>>,
) -> String {
    let start = line.find("{{>").unwrap() + 3;
    let end = line.find("}}").unwrap();
    let partial_name = line[start..end].trim();

    if let Some(partial_lines) = partials.get(partial_name) {
        let mut output = String::new();
        for partial_line in partial_lines {
            output.push_str(&interpolate_variables(partial_line, context));
            output.push('\n');
        }
        output
    } else {
        format!("/* partial not found: {} */", partial_name)
    }
}

fn interpolate_variables(line: &str, context: &TemplateContext) -> String {
    let mut result = line.to_string();

    // Simple variable replacement {{variable}}
    while let Some(start) = result.find("{{") {
        if let Some(end) = result[start..].find("}}") {
            let end = start + end + 2;
            let var_expr = &result[start + 2..end - 2].trim();

            // Skip control structures
            if var_expr.starts_with('#') || var_expr.starts_with('/') || var_expr.starts_with('>') {
                break;
            }

            let value = evaluate_expression(var_expr, context);
            result = format!("{}{}{}", &result[..start], value, &result[end..]);
        } else {
            break;
        }
    }

    result
}

fn evaluate_expression(expr: &str, context: &TemplateContext) -> String {
    // Handle helper functions
    if expr.starts_with("snake_case ") {
        let arg = &expr[11..];
        let value = get_variable(arg, context);
        return to_snake_case(&value);
    }
    if expr.starts_with("pascal_case ") {
        let arg = &expr[12..];
        let value = get_variable(arg, context);
        return to_pascal_case(&value);
    }
    if expr.starts_with("screaming_snake_case ") {
        let arg = &expr[21..];
        let value = get_variable(arg, context);
        return to_screaming_snake_case(&value);
    }
    if expr.starts_with("rust_type ") {
        let arg = &expr[10..];
        return get_variable(arg, context);
    }

    // Simple variable lookup
    get_variable(expr, context)
}

fn get_variable(name: &str, context: &TemplateContext) -> String {
    match name {
        "version" => context.version.clone(),
        "name" => context
            .variables
            .get("name")
            .cloned()
            .unwrap_or_default(),
        _ => context
            .variables
            .get(name)
            .cloned()
            .unwrap_or_else(|| format!("/* unknown: {} */", name)),
    }
}

fn evaluate_condition(condition: &str, context: &TemplateContext) -> bool {
    match condition {
        "optional" => context
            .variables
            .get("optional")
            .map(|v| v == "true")
            .unwrap_or(false),
        "required" => context
            .variables
            .get("required")
            .map(|v| v == "true")
            .unwrap_or(false),
        _ => context.variables.contains_key(condition),
    }
}

fn create_type_context(type_info: &TypeInfo, base: &TemplateContext) -> TemplateContext {
    let mut ctx = TemplateContext {
        version: base.version.clone(),
        ..Default::default()
    };
    ctx.variables.insert("name".to_string(), type_info.name.clone());
    ctx
}

fn create_endpoint_context(endpoint_info: &EndpointInfo, base: &TemplateContext) -> TemplateContext {
    let mut ctx = TemplateContext {
        version: base.version.clone(),
        ..Default::default()
    };
    ctx.variables
        .insert("name".to_string(), endpoint_info.name.clone());
    ctx.variables
        .insert("method".to_string(), endpoint_info.method.clone());
    ctx.variables
        .insert("path".to_string(), endpoint_info.path.clone());
    ctx
}

fn interpolate_path(path: &str, context: &TemplateContext) -> String {
    interpolate_variables(path, context)
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn to_screaming_snake_case(s: &str) -> String {
    to_snake_case(s).to_uppercase()
}
