use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;
use std::fs;
use std::path::Path;

// [意図] コマンドテンプレート内のプレースホルダー（$TEXT, {{text}}）を、
// フロントエンドからの入力テキストで安全に置換する。
pub fn replace_variables(input: &str, text: &str) -> String {
    input.replace("$TEXT", text).replace("{{text}}", text)
}

// [意図] OS環境変数（HOME/USERPROFILE）を使用して、~ を絶対パスへ変換する。
// WindowsとUnix系両対応のため、環境変数のフォールバックを利用して安全にパスを解決する。
pub fn append_to_file(path: &str, text: &str) -> Result<(), String> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).map_err(|_| "Could not find home directory")?;
    let full_path = path.replace("~", &home);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&full_path)
        .map_err(|e| format!("Failed to open {}: {}", full_path, e))?;

    writeln!(file, "{}", text).map_err(|e| e.to_string())
}

// [意図] 送信履歴をJSONとしてファイルに保存し、永続化する。
pub fn save_history(history: &str, config_dir: &str) -> Result<(), String> {
    let path = Path::new(config_dir);
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| e.to_string())?;
    }
    let history_path = path.join("hookline_history.json");
    fs::write(history_path, history).map_err(|e| e.to_string())
}

// [意図] 送信履歴をファイルから読み込む。
pub fn load_history(config_dir: &str) -> Result<String, String> {
    let history_path = Path::new(config_dir).join("hookline_history.json");
    if !history_path.exists() {
        return Ok("[]".to_string());
    }
    fs::read_to_string(history_path).map_err(|e| e.to_string())
}

// [意図] 設定情報をJSONとして保存する（プロファイル、テーマ、位置情報等）。
pub fn save_settings(data: &str, config_dir: &str) -> Result<(), String> {
    let path = Path::new(config_dir);
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| e.to_string())?;
    }
    let settings_path = path.join("settings.json");
    fs::write(settings_path, data).map_err(|e| e.to_string())
}

// [意図] 設定情報をファイルから読み込む。存在しない場合やプロファイルが空の場合はサンプルを返す。
pub fn load_settings(config_dir: &str) -> Result<String, String> {
    let settings_path = Path::new(config_dir).join("settings.json");
    
    let default_settings = serde_json::json!({
        "profiles": [
            {"id": 1, "name": "Daily Journal", "target": "~/journal.md", "command": "echo %TEXT% >> %FILE%"}
        ],
        "isDarkMode": false,
        "windowPos": {"x": 0, "y": 0}
    });

    if !settings_path.exists() {
        return Ok(default_settings.to_string());
    }

    let data = fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;
    let mut json: serde_json::Value = serde_json::from_str(&data).map_err(|e| e.to_string())?;

    // プロファイルが空ならサンプルを追加
    if let Some(profiles) = json.get_mut("profiles").and_then(|p| p.as_array_mut()) {
        if profiles.is_empty() {
            *profiles = default_settings["profiles"].as_array().unwrap().clone();
        }
    }
    
    Ok(json.to_string())
}

pub fn execute_command(cmd_template: &str, text: &str) -> Result<String, String> {
    // [意図] OSごとのシェル実行環境の違いを吸収し、ユーザー指定コマンドを安全に実行する。
    // Windowsはcmd、Unix系はshを経由し、任意のコマンド実行を可能にする。
    // テンプレート内の変数を置換
    let replaced_cmd = replace_variables(cmd_template, text);

    // OS判定により実行コマンドを切り替え
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", &replaced_cmd])
        .output();

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("sh")
        .args(["-c", &replaced_cmd])
        .output();

    match output {
        Ok(out) if out.status.success() => Ok(String::from_utf8_lossy(&out.stdout).to_string()),
        Ok(out) => Err(format!("Command failed: {}", String::from_utf8_lossy(&out.stderr))),
        Err(e) => Err(e.to_string()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;

    #[test]
    fn test_replace_variables() {
        assert_eq!(replace_variables("echo $TEXT", "hello"), "echo hello");
        assert_eq!(replace_variables("echo {{text}}", "world"), "echo world");
        assert_eq!(replace_variables("no variable", "test"), "no variable");
        assert_eq!(replace_variables("$TEXT and {{text}}", "dup"), "dup and dup");
    }

    #[test]
    fn test_append_to_file_logic() {
        // テスト用の一時ファイルを作成
        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join("test_append.txt");
        let path_str = file_path.to_str().unwrap();

        // 追記テスト
        let _ = append_to_file(path_str, "line 1");
        let _ = append_to_file(path_str, "line 2");

        // 内容の検証
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "line 1\nline 2\n");

        // 後始末
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_execute_command_echo() {
        // echoコマンドを使って、コマンド実行と置換が正常に行われるかテスト
        // OSごとにechoコマンドの挙動が異なる可能性があるため、シンプルな形式で実行
        let result = execute_command("echo {{text}}", "test_message");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "test_message");
    }

    #[test]
    fn test_save_history_logic() {
        // 実際の履歴データの構造（id, profile, text, timestamp）と一致するテストデータ
        let history_json = r#"[{"id":1746589200000,"profile":"Daily Journal","text":"hello","timestamp":1746589200000}]"#;

        // テスト用の一時ディレクトリを取得
        let temp_dir = env::temp_dir();
        let config_dir = temp_dir.to_str().unwrap();

        // 履歴保存テスト
        let result = save_history(history_json, config_dir);
        assert!(result.is_ok());

        // 保存されたファイルの読み込みと検証
        let history_path = Path::new(config_dir).join("hookline_history.json");
        let content = fs::read_to_string(&history_path).unwrap();
        assert_eq!(content, history_json);

        // 後始末
        let _ = fs::remove_file(history_path);
    }
    }