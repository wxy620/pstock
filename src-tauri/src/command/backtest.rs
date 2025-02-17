use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

///
///
/// demo 调用 python backtrader 进行回测， 出现错误 raise一个error
/// 如果正常
/// 1、 输出结果到目录比如, mac /Users/niko/Download/mystock/backtest/_id/中
/// 2、 文件格式为 parquet，csv。 用 polars 进行解析
/// 3、 如果quantstats 稳定，可以提供html，作为结果
///
///
#[tauri::command]
pub async fn call_my_sidecar(app: tauri::AppHandle) {
    let sidecar_command = app
        .shell()
        .sidecar("main")
        .unwrap();
        // .args(["arg1", "-a", "--arg2", "any-string-that-matches-the-validator"]);
    let (mut rx, mut child) = sidecar_command
        .spawn()
        .expect("Failed to spawn sidecar");

    tauri::async_runtime::spawn(async move {
        // read events such as stdout
        while let Some(event) = rx.recv().await {
            match  event {
                CommandEvent::Stdout(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    println!("{}", line);
                    // window
                    //     .emit("message", Some(format!("'{}'", line)))
                    //     .expect("failed to emit event");
                    // write to stdin
                    child.write("message from Rust\n".as_bytes()).unwrap();
                },
                CommandEvent::Stderr(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    println!("{:#?}", line);
                },
                _=>()
            }

        }
    });

}