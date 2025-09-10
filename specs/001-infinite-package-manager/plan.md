## **機能実装計画：Infinite Package Manager**

### **現在の進捗状況:**

*   **CLIフレームワーク:** `clap` が引数解析のために設定されており、基本的なサブコマンド（`License`、`Manual`、`Pkg`、`Repo`）が定義されています。
*   **モジュール構造:** `src/modules/compats/pkg` と `src/modules/compats/repo` は、それぞれパッケージマネージャーとリポジトリの互換性のために指定されています。
*   **プレースホルダーファイル:** `dpkg.rs`、`ipak.rs`、`apt.rs` が存在し、特定のパッケージマネージャー/リポジトリの統合を実装する意図を示しています。
*   **リポジトリの抽象化:** `src/modules/compats/repo.rs` で `Repository` 構造体と `RepoType` 列挙型が定義されており、リポジトリ管理のための計画的な抽象化が示唆されています。

### **必要な実装:**

1.  **汎用パッケージマネージャーインターフェース:**
    *   `src/modules/compats/pkg.rs` に、標準的なパッケージ管理操作（例: `install`、`remove`、`list`、`search`）の概要を定義する共通の `trait`（インターフェース）を定義します。すべての特定のパッケージマネージャー実装（dpkg、rpm、pacman）は、このトレイトを実装します。

2.  **特定のパッケージマネージャー実装:**
    *   **`src/modules/compats/pkg/dpkg.rs`**: `dpkg` の `PackageManager` トレイトを実装します。これには、`dpkg` シェルコマンドの実行とその出力の解析が含まれます。
    *   **他のパッケージマネージャー用の新規ファイル**: `src/modules/compats/pkg/` 内に `rpm.rs`、`pacman.rs` などを作成し、それぞれのコマンドライン操作を処理する `PackageManager` トレイトを実装します。

3.  **`Pkg` コマンドロジック（`src/modules/command/pkg.rs`）:**
    *   **パッケージマネージャーの検出:** ユーザーのシステムにどのパッケージマネージャーが存在するかを検出するロジックを実装します。
    *   **コマンドのディスパッチ:** ユーザー入力（例: `pkg install <package>`）と検出されたパッケージマネージャーに基づいて、適切な `PackageManager` 実装に操作をディスパッチします。
    *   **結果の集約:** `pkg list` のようなコマンドの場合、検出されたすべてのパッケージマネージャーからの結果を統合された形式で集約して表示します。
    *   **サブコマンドの定義:** `src/utils/args.rs` を更新して、`pkg` のサブコマンド（例: `install`、`remove`、`list`、`search`）を定義します。

4.  **リポジトリ管理（`src/modules/command/repo.rs` および `src/modules/compats/repo/apt.rs`）:**
    *   パッケージ管理と同様に、`apt` リポジトリの特定のロジック（ソースの追加、削除、更新）を実装します。
    *   他のリポジトリタイプ（例: yum、pacman）がサポートされる場合は、汎用 `RepositoryManager` トレイトを定義します。

5.  **エラー処理の強化（`src/utils/error.rs`）:**
    *   パッケージマネージャーの操作に関連するより具体的なエラータイプ（例: `PackageManagerNotFound`、`CommandExecutionError`、`PackageNotFound`）を追加します。

### **今後の開発ステップ:**

1.  `src/modules/compats/pkg.rs` に **`PackageManager` トレイトを定義**します。
2.  `src/modules/compats/pkg/dpkg.rs` に、`PackageManager` トレイトを実装する構造体を作成して、**`dpkg` 統合を実装**します。
3.  `src/utils/args.rs` を更新して、`pkg` コマンドに必要なサブコマンドと引数を含めます。
4.  `src/modules/command/pkg.rs` の `run` 関数を具体化し、`dpkg` 操作のために `PackageManager` トレイトを利用するようにします。