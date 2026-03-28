use crate::print_ops::enqueue_print;
use cli_shared::TemplateArgs;

pub async fn handle_template_command(args: TemplateArgs, cut: bool) -> anyhow::Result<String> {
    let task = args.command.into_print_task(cut)?;
    enqueue_print(task).await;
    Ok("Template printed successfully.".to_string())
}
