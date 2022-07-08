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
  *  Fix version in package.json ([d47afc64](d47afc64))
  *  Escape & symbols in HTML ([856fd66b](856fd66b))
  *  Declare HTML encoding ([e5b1ff1e](e5b1ff1e))
* **host:**  Use std::current_exe() to get program path ([34741e25](34741e25))

#### Features

*   Add option to bypass version check ([4f752ba1](4f752ba1))
*   Initial working copy ([f40ea892](f40ea892))
* **ext:**
  *  Replace shell select with input ([97242477](97242477))
  *  Use Homebrew binaries on macOS ([efb67969](efb67969))
* **host:**
  *  Escape file name under Windows ([86a91d93](86a91d93))
  *  Allow HTML body ([51c70f63](51c70f63))
  *  Print notification contents to stderr ([f9be4d51](f9be4d51))
  *  Handle editor process exit status ([10ec54bd](10ec54bd))
  *  Add macOS manifest location hint ([23ecd8cd](23ecd8cd))
  *  Print manifest help to stderr ([58b0b672](58b0b672))
  *  Clean up temporary files automatically ([09a1fe64](09a1fe64))
