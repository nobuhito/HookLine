[English](README.md) | 日本語

# HookLine

HookLineは、テキストを入力して「ファイルへの追記」や「外部コマンドの実行」を瞬時に行うための、常駐型生産性向上ツールです。WSLでの堅牢なロジック開発と、Windows上での軽快なデスクトップ体験を両立させています。

## 主な機能
- **ファイル追記:** 入力テキストを任意のファイル末尾に改行付きで追記。
- **外部コマンド実行:** 変数 `$TEXT` または `{{text}}` を用いて、シェルコマンドを安全に実行。
- **マルチプロファイル:** 用途に合わせて最大9つまでの設定を切り替え可能。
- **送信履歴:** 過去の送信内容を保存し、素早く呼び出し可能。
- **グローバルショートカット:** `Alt + Space` でいつでも即座に起動。

## 操作方法
| キー操作 | アクション |
| :--- | :--- |
| `Alt + Space` | ウィンドウの表示 / 非表示 |
| `Ctrl + 1 〜 9` | プロファイルの切り替え |
| `Enter` | 送信 (追記 & コマンド実行) |
| `Shift + Enter` | 入力欄内での改行 |
| `Esc` | ウィンドウを隠す |

## プロファイル設定サンプル
初期設定として以下のサンプルが含まれています。
- **Daily Journal**
  - Target: `~/journal.md`
  - Command: `echo %TEXT% >> %FILE%`

## スクリーンショット
*(ここにアプリのスクリーンショットが入ります)*

## 技術スタック
- **Backend:** Rust (Tauri v2)
- **Frontend:** Vue.js 3 (Composition API) + Vite

## ライセンス
本プロジェクトは MIT ライセンスの下で公開されています。

## 支援のお願い
HookLine の開発を継続するため、もし気に入っていただけましたらコーヒー1杯分($3 / 500円)の支援をいただけると励みになります！

[<img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me a Coffee" style="height: 60px !important;width: 217px !important;" >](https://www.buymeacoffee.com/nobuhito)
