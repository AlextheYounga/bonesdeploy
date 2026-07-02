use anyhow::Result;
use serde_json::Value;
use shared::config::{bonesinfra_input, runtime_group_for, runtime_user_for};

use super::RuntimeSelection;
use crate::infra::bonesinfra;
use crate::infra::embedded;
use crate::ui::prompts;

pub(super) fn collect_runtime_config(args: &super::Args, project_name: &str) -> Result<RuntimeSelection> {
    let template = if args.non_interactive {
        None
    } else {
        let available = embedded::runtime_names();
        prompts::choose_template(&available)?
    };

    if let Some(ref template_name) = template {
        let defaults = embedded::runtime_defaults(template_name)?;
        let answers = if args.non_interactive {
            Value::Object(defaults.clone())
        } else {
            let questions = bonesinfra::runtime_questions(template_name)?;
            prompts::prompt_runtime_questions(&questions, &Value::Object(defaults.clone()))?
        };
        let mut map = answers.as_object().cloned().unwrap_or(defaults);
        inject_runtime_identity(&mut map, project_name);
        Ok(RuntimeSelection { template: Some(template_name.clone()), config: map })
    } else {
        let mut vars = embedded::base_runtime_defaults()?;
        inject_runtime_identity(&mut vars, project_name);
        Ok(RuntimeSelection { template: None, config: vars })
    }
}

fn inject_runtime_identity(vars: &mut serde_json::Map<String, Value>, project_name: &str) {
    vars.insert(bonesinfra_input::RUNTIME_USER.into(), Value::String(runtime_user_for(project_name)));
    vars.insert(bonesinfra_input::RUNTIME_GROUP.into(), Value::String(runtime_group_for(project_name)));
}
