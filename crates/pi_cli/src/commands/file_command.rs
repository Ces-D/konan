use crate::print_ops::enqueue_print;
use cli_shared::{PrintTask, file_command::FileArgs, tasks::KonanFile};

pub async fn handle_file_command(args: FileArgs, cut: bool) -> anyhow::Result<String> {
    let name = args
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| args.path.to_string_lossy().into_owned());
    enqueue_print(PrintTask::File(KonanFile {
        name,
        cut,
        rows: args.rows,
        prehook_command: args.prehook_command,
        prehook_command_arg: args.prehook_command_args,
    }))
    .await;
    Ok("File printed successfully.".to_string())
}
