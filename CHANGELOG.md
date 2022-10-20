<a name="v0.4.2"></a>
## v0.4.2 (2022-10-20)


#### Bug Fixes

* **host:**  Fix crashes when EML has invalid UTF-8 ([c8a428d0](https://github.com/Frederick888/external-editor-revived/commit/c8a428d005b0e0c69c5302868d1e7858873c1692))



<a name="v0.4.1"></a>
## v0.4.1 (2022-09-27)


#### Bug Fixes

* **host:**  Support both LF & CRLF under Windows ([49be80be](https://github.com/Frederick888/external-editor-revived/commit/49be80bebb976c4e18c0458c6f465ae5085bacca))



<a name="v0.4.0"></a>
## v0.4.0 (2022-08-03)


#### Features

* **ext:**  Tooltip for compose window button ([513bb962](https://github.com/Frederick888/external-editor-revived/commit/513bb962c795226d269d5127631f69670faedc7c))
* **host:**
  *  CRLF in temporary EML files ([31da04d2](https://github.com/Frederick888/external-editor-revived/commit/31da04d2c67b9b35a9bdc303bee0cf2a358bf854))
  *  Add placeholders for absent headers ([86f59dbd](https://github.com/Frederick888/external-editor-revived/commit/86f59dbd344b521d7dce91397aa2c67335ee7e91))
  *  Conciser help headers at the bottom ([18ef80d0](https://github.com/Frederick888/external-editor-revived/commit/18ef80d087da759f3338de516300fd92ca1ce3e2))
  *  Skip checking patch version ([eeacaa65](https://github.com/Frederick888/external-editor-revived/commit/eeacaa65ad974b1a2fa8147f827ba9d1d688a657))

#### Bug Fixes

* **ext:**  Use correct identity to reply ([505393c4](https://github.com/Frederick888/external-editor-revived/commit/505393c4de00ce400ffd1d461fc6331eb7df0351))
* **host:**  Stop unnecessarily trimming email bodies ([1692bef1](https://github.com/Frederick888/external-editor-revived/commit/1692bef15d6fb9d62323d3e35331e9421a98fb3b))



<a name="v0.3.0"></a>
## v0.3.0 (2022-07-15)


#### Features

*   Shift-click on buttons for Send-On-Exit ([533358dd](https://github.com/Frederick888/external-editor-revived/commit/533358dd62553390ea5306f82bf2ad4bd24abc95))
*   Cancel send-on-exit in case of unknown headers ([3ddc4609](https://github.com/Frederick888/external-editor-revived/commit/3ddc4609ab73308503ad31a19e5aab75450a2a30))
* **ext:**
  *  Notifications for sendMessage() errors ([a1536f10](https://github.com/Frederick888/external-editor-revived/commit/a1536f10d7129edc8b5678aef4cd1196bfc8d512))
  *  Commands for reply from mail tab ([54b9e7d5](https://github.com/Frederick888/external-editor-revived/commit/54b9e7d51ccd9ff13a8483f986bf774a6c0e41bc))
  *  Create message commands without default shortcuts ([81c84b6b](https://github.com/Frederick888/external-editor-revived/commit/81c84b6bd5ad089bf34aec76b61293bdaf458e48))
  *  Disable compose action when waiting for editor ([8554f39e](https://github.com/Frederick888/external-editor-revived/commit/8554f39e81650f89ca980adba152a062e8a98085))
  *  Notifications for messaging host errors ([d313ad4b](https://github.com/Frederick888/external-editor-revived/commit/d313ad4b98bced321cc4043cdc6b2cfb11d68758))
  *  Ctrl-Shift-E to edit with Send-On-Exit ([2cf4f040](https://github.com/Frederick888/external-editor-revived/commit/2cf4f0409604bf202095d8f571b9eaf283590054))
  *  Add main toolbar button (browserAction) ([a47c3dd9](https://github.com/Frederick888/external-editor-revived/commit/a47c3dd94128dd3f4b2d7753b5da1ba3657d7669))
  *  Adjust wording of version bypass description ([22d7774d](https://github.com/Frederick888/external-editor-revived/commit/22d7774de494dd10899ac80903070b8f57b1aa85))
* **host:**  Print target OS & arch in -v output ([8f1155fd](https://github.com/Frederick888/external-editor-revived/commit/8f1155fd16103e0b4c1f87aafdc876db437a1309))

#### Bug Fixes

* **ext:**  Small fix for header help message ([ade84119](https://github.com/Frederick888/external-editor-revived/commit/ade8411953b92f8330f5fb3a063968349d4ff096))



<a name="v0.2.0"></a>
## v0.2.0 (2022-07-10)


#### Bug Fixes

* **ext:**
  *  Template cleared after selecting Custom ([cd69617f](https://github.com/Frederick888/external-editor-revived/commit/cd69617f20ea8ce386c8da0cc7d521aff1d0bf7b))
  *  Not concatenating chunked HTML body ([175bc5f9](https://github.com/Frederick888/external-editor-revived/commit/175bc5f9df168870fb944cfc91fc0815e066bd2c))
  *  Editor command should use Homebrew path in macOS ([5f8e6ae8](https://github.com/Frederick888/external-editor-revived/commit/5f8e6ae828a1abebebcce9c04fa13d6d39be9371))

#### Features

*   Add help message headers ([2e1f9e15](https://github.com/Frederick888/external-editor-revived/commit/2e1f9e15af5ed06ee36ba4574803c2f15d36295d))
* **ext:**
  *  Add Ctrl-E shortcut for compose action ([0569819e](https://github.com/Frederick888/external-editor-revived/commit/0569819eec18ab1ef8268168b37a6c99053a7824))
  *  Use textarea for templates ([9d5b0b17](https://github.com/Frederick888/external-editor-revived/commit/9d5b0b1782259b9d3ece7e52a95798db9e17eafb))
  *  Prompt to sync with upstream template ([86763e6d](https://github.com/Frederick888/external-editor-revived/commit/86763e6d5219df8833db93ff4d8b1f804bc80d02))
  *  Kitty macos_quit_when_last_window_closed ([3d802fcc](https://github.com/Frederick888/external-editor-revived/commit/3d802fcc3c40dd3113ebb8492ca289daba5c5890))
* **host:**
  *  Make header names case-insensitive ([e7dab336](https://github.com/Frederick888/external-editor-revived/commit/e7dab336b34d89bf29211ebd617aee909cc199cc))
  *  Rename Send-On-Save to Send-On-Exit ([f22f00ad](https://github.com/Frederick888/external-editor-revived/commit/f22f00ad3e9acc35513d754938de5eb3395876d8))
  *  Skip relatedMessageId if null ([b580bbaa](https://github.com/Frederick888/external-editor-revived/commit/b580bbaa9285ee20792025f203b884d7a2b26823))
  *  Add -i, -l to macOS shell arguments ([7c119a09](https://github.com/Frederick888/external-editor-revived/commit/7c119a0938b78120ae6ca06cbec5431989289304))
  *  Support -v/-h for version/help message ([f4bde758](https://github.com/Frederick888/external-editor-revived/commit/f4bde7582cabf5a5fb8378c06d7ed533ff642b13))



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
