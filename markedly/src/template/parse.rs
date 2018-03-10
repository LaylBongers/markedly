use pest::iterators::{Pair};

use template::{ComponentTemplate, TemplateAttribute, TemplateValue};

#[derive(Parser)]
#[grammar = "template/language.pest"]
pub struct TemplateParser;

pub fn parse_document(document_pair: Pair<Rule>) -> Result<Vec<ComponentTemplate>, String> {
    assert_eq!(document_pair.as_rule(), Rule::template);

    let mut components = Vec::new();

    let mut parent_stack: Vec<ComponentTemplate> = Vec::new();
    let mut last_indentation = 0;
    for pair in document_pair.into_inner() {
        let (component, indentation) = parse_component(pair.clone())?;

        // Prevent first component starting at wrong indentation level
        if components.len() == 0 && parent_stack.len() == 0 {
            if indentation != 0 {
                return Err("First component starts at wrong indentation".into())
            }
        }

        // If we're at the same indentation level as the previous component,
        // the previous component is our sibling, not parent
        if indentation == last_indentation {
            finish_sibling(&mut parent_stack, &mut components);
        }

        // If we are at lower indentation level, unroll the stack to the level we need to be at
        if indentation < last_indentation {
            let unroll_amount = last_indentation - indentation + 1;
            for _ in 0..unroll_amount {
                finish_sibling(&mut parent_stack, &mut components);
            }
        }

        // If our indentation has increased by more than one, we need to give an error for that
        if indentation > last_indentation && indentation - last_indentation > 1 {
            let (line, _col) = pair.into_span().start_pos().line_col();
            return Err(format!("Excessive increase in indentation at line {}", line))
        }

        parent_stack.push(component);
        last_indentation = indentation;
    }

    // Unroll the stack into a final component
    let mut last_component = None;
    parent_stack.reverse();
    for mut component in parent_stack {
        if let Some(child_component) = last_component.take() {
            component.children.push(child_component);
        }
        last_component = Some(component);
    }
    if let Some(component) = last_component {
        components.push(component);
    }

    Ok(components)
}

fn finish_sibling(
    parent_stack: &mut Vec<ComponentTemplate>, components: &mut Vec<ComponentTemplate>
) {
    // If we don't have anything above us on the parent stack, that means we're the very
    // first component in the file, so there's no previous component to add to anything
    if let Some(sibling) = parent_stack.pop() {
        // If we don't have a parent for the sibling, that means we're at the root of the
        // file, so instead it needs to be added to the final components list
        if let Some(mut parent) = parent_stack.pop() {
            // However if both of those things are not the case, just add it to our parent
            parent.children.push(sibling);
            parent_stack.push(parent);
        } else {
            components.push(sibling);
        }
    }
}

fn parse_component(pair: Pair<Rule>) -> Result<(ComponentTemplate, usize), String> {
    assert_eq!(pair.as_rule(), Rule::component);
    let mut indentation = 0;
    let mut class = None;
    let mut style_class: Option<String> = None;
    let mut attributes = None;
    let (line, _col) = pair.clone().into_span().start_pos().line_col();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::indentation => indentation = parse_indentation(pair)?,
            Rule::identifier => class = Some(pair.as_str().into()),
            Rule::style_class => style_class = Some(pair.as_str()[1..].into()),
            Rule::attributes => attributes = Some(parse_attributes(pair)?),
            _ => {}
        }
    }

    Ok((ComponentTemplate {
        class: class.unwrap(),
        style_class,
        attributes: attributes.unwrap_or_else(|| Vec::new()),
        children: Vec::new(),
        line,
    }, indentation))
}

fn parse_indentation(pair: Pair<Rule>) -> Result<usize, String> {
    // Count the spacing, including tabs
    let mut spacing = 0;
    for c in pair.as_str().chars() {
        match c {
            ' ' => spacing += 1,
            '\t' => spacing += 4,
            _ => unreachable!(),
        }
    }

    // Fail indentation that isn't divisible by 4
    if spacing % 4 != 0 {
        let (line, _col) = pair.into_span().start_pos().line_col();
        return Err(format!("Bad amount of indentation spacing, must be divisible by 4, at line {}", line))
    }

    Ok(spacing/4)
}

fn parse_attributes(pair: Pair<Rule>) -> Result<Vec<TemplateAttribute>, String> {
    assert_eq!(pair.as_rule(), Rule::attributes);

    let mut attributes: Vec<TemplateAttribute> = Vec::new();

    for key_value_pair in pair.into_inner() {
        assert_eq!(key_value_pair.as_rule(), Rule::key_value);

        let mut key: Option<String> = None;
        let mut value: Option<TemplateValue> = None;
        let mut script_conditional: Option<String> = None;

        for pair in key_value_pair.clone().into_inner() {
            match pair.as_rule() {
                Rule::identifier =>
                    key = Some(pair.as_str().into()),
                Rule::value =>
                    value = Some(parse_value(pair)),
                Rule::script_conditional => {
                    let pair_str = pair.as_str();
                    script_conditional = Some(pair_str[2..pair_str.len()-1].into());
                }
                _ => unreachable!(),
            }
        }

        // We allow duplicate keys, when attributes are resolved it will pick the last one
        attributes.push(TemplateAttribute {
            key: key.unwrap(),
            value: value.unwrap(),
            script_conditional,
        });
    }

    Ok(attributes)
}

fn parse_value(pair: Pair<Rule>) -> TemplateValue {
    assert_eq!(pair.as_rule(), Rule::value);
    let pair = pair.into_inner().next().unwrap();

    let pair_str = pair.as_str();
    match pair.as_rule() {
        Rule::string =>
            TemplateValue::String(pair_str[1..pair_str.len()-1].into()),
        Rule::percentage =>
            TemplateValue::Percentage(pair_str[0..pair_str.len()-1].parse().unwrap()),
        Rule::integer =>
            TemplateValue::Integer(pair_str.parse().unwrap()),
        Rule::float =>
            TemplateValue::Float(pair_str.parse().unwrap()),
        Rule::tuple => {
            let mut values = Vec::new();
            for pair in pair.into_inner() {
                values.push(parse_value(pair));
            }
            TemplateValue::Tuple(values)
        },
        Rule::default =>
            TemplateValue::Default,
        Rule::script_value =>
            TemplateValue::ScriptValue(pair_str[2..pair_str.len()-1].into()),
        Rule::script_statement =>
            TemplateValue::ScriptStatement(pair_str[1..pair_str.len()-1].into()),
        _ => unreachable!(),
    }
}
