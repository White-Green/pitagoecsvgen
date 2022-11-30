# ぴた声アプリ用設定csvジェネレータ

[ぴた声アプリ](https://www.ah-soft.com/trial/pitagoe.html) ([document](https://www.ah-soft.com/pitagoe/app/guide/)) で利用する設定csvファイルの作成をweb上で行うツールです。

# 開発方法

[Rust](https://www.rust-lang.org/ja/) ([download](https://rustup.rs)) 及び [wasm-pack](https://github.com/rustwasm/wasm-pack) 、[nodejs](https://nodejs.org/ja/) 、 [yarn](https://yarnpkg.com/) (必須ではないが利用しない場合コマンドの読み替えが必要)が必要です

プロジェクトのセットアップ
```shell
$ git clone https://github.com/White-Green/pitagoegen
$ cd pitagoegen
$ cd pitagoegen_core
$ ./build.ps1 # もしくは build.ps1内記述のコマンド
$ cd ..
$ yarn
```

開発用サーバの起動
```shell
$ yarn start
```

リリースビルド
```shell
$ yarn build
```

GitHub Pagesデプロイ(自分用メモ)
```shell
$ git subtree push --prefix build origin gh-pages
```