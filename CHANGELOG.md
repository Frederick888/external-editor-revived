<a name="v0.1.1"></a>
## v0.1.1 (2022-07-08)

No code changes.

This is for the sake of addons.thunderbird.net. I uploaded 0.1.0 earlier
when experimenting and it doesn't allow me to upload 0.1.0 again even
after I deleted the 0.1.0 draft.



<a name="v0.1.0"></a>
## v0.1.0 (2022-07-08)


#### Bug Fixes

* **ext:**
  *  Fix version in package.json ([d47afc64](https://github.com/Frederick888/external-editor-revived/commit/d47afc6491e69b3177f5a764dd8872be3c4d42a9))
  *  Escape & symbols in HTML ([856fd66b](https://github.com/Frederick888/external-editor-revived/commit/856fd66b032b5c48a56d7ac2a5791bbcc10b61ac))
  *  Declare HTML encoding ([e5b1ff1e](https://github.com/Frederick888/external-editor-revived/commit/e5b1ff1ea01d3d6268232f9081ba828f2043919a))
* **host:**  Use std::current_exe() to get program path ([34741e25](https://github.com/Frederick888/external-editor-revived/commit/34741e25ca2b9f8007b64c87458a1b6355b709f7))

#### Features

*   Add option to bypass version check ([4f752ba1](https://github.com/Frederick888/external-editor-revived/commit/4f752ba1ed9df4af4243cf8a2dba995840054a4e))
*   Initial working copy ([f40ea892](https://github.com/Frederick888/external-editor-revived/commit/f40ea892c3bc7f467278d41b482e240b367cd8db))
* **ext:**
  *  Replace shell select with input ([97242477](https://github.com/Frederick888/external-editor-revived/commit/9724247798c0f8f4dddfb280ccf0ce5de8b0c28d))
  *  Use Homebrew binaries on macOS ([efb67969](https://github.com/Frederick888/external-editor-revived/commit/efb679693a26306fcbc4d2447025d10ca6615ec6))
* **host:**
  *  Escape file name under Windows ([86a91d93](https://github.com/Frederick888/external-editor-revived/commit/86a91d938e50851e6d7727d089d0872eb3f74032))
  *  Allow HTML body ([51c70f63](https://github.com/Frederick888/external-editor-revived/commit/51c70f63cd01ff7f00f62c36835696845e6a3109))
  *  Print notification contents to stderr ([f9be4d51](https://github.com/Frederick888/external-editor-revived/commit/f9be4d51d560ccddf6bf6e37d35f61f183583d8c))
  *  Handle editor process exit status ([10ec54bd](https://github.com/Frederick888/external-editor-revived/commit/10ec54bd3058a306a039664e2f4b23638700bb3a))
  *  Add macOS manifest location hint ([23ecd8cd](https://github.com/Frederick888/external-editor-revived/commit/23ecd8cdcda047943e9933d849b194ce14171827))
  *  Print manifest help to stderr ([58b0b672](https://github.com/Frederick888/external-editor-revived/commit/58b0b672e747b7be3aac63f7866a803a607e0fdf))
  *  Clean up temporary files automatically ([09a1fe64](https://github.com/Frederick888/external-editor-revived/commit/09a1fe64bb5b8df26fc1e0b1156cbbc043074596))
