class Editor {
  constructor(command, gui) {
    this.command = command
    this.gui = gui
  }
}
const editors = {
  'nvim': new Editor('nvim', false),
  'vim': new Editor('vim', false),
  'emacs': new Editor('emacs', true),
  'kak': new Editor('kak', false),
  'neovide': new Editor('neovide --nofork', true),
  'gvim': new Editor('gvim --nofork', true),
}
const homebrewDefaultDir = '/usr/local/bin/'

const editorSelect = document.getElementById('editor')
const terminalRow = document.getElementById('terminal-row')
const terminalSelect = document.getElementById('terminal')
const shellRow = document.getElementById('shell-row')
const shellSelect = document.getElementById('shell')
const templateInput = document.getElementById('template')
const bypassVersionCheckInput = document.getElementById('bypass-version-check')
const applyButton = document.getElementById('apply')

function updateOptionsForEditor(editor) {
  if (editor === 'custom') {
    showElement(shellRow)
    templateInput.removeAttribute('disabled')
    hideElement(terminalRow)
  } else {
    hideElement(shellRow)
    templateInput.setAttribute('disabled', 'true')
    const editorConfig = editors[editor]
    if (editorConfig.gui) {
      hideElement(terminalRow)
    } else {
      showElement(terminalRow)
    }
  }
}

editorSelect.onchange = async (e) => {
  const editor = e.target.value
  updateOptionsForEditor(editor)
  await updateTemplate()
}
terminalSelect.onchange = async () => {
  await updateTemplate()
}

let applyButtonCountdown = null
applyButton.onclick = async () => {
  if (applyButtonCountdown !== null) {
    clearInterval(applyButtonCountdown)
  }
  applyButtonCountdown = setInterval(() => {
    applyButton.setAttribute('value', 'Apply')
    applyButton.removeAttribute('disabled')
  }, 750)
  applyButton.setAttribute('value', 'Saved!')
  applyButton.setAttribute('disabled', 'true')
  await saveSettings()
}

function hideElement(element) {
  element.style = 'display: none;'
}

function showElement(element) {
  element.style = ''
}

const templateTempFileName = '"/path/to/temp.eml"'
async function updateTemplate() {
  const editor = editorSelect.value
  if (editor === 'custom') {
    return
  }

  const platform = await browser.runtime.getPlatformInfo()
  const editorConfig = editors[editor]
  const editorCommand = platform.os === browser.runtime.PlatformOs.MAC ? homebrewDefaultDir + editorConfig.command : editorConfig.command
  if (editorConfig.gui) {
    templateInput.value = editorCommand + " " + templateTempFileName
    return
  }

  let terminalCommand = platform.os === browser.runtime.PlatformOs.MAC ? homebrewDefaultDir : ''
  switch (terminalSelect.value) {
    case 'kitty':
      terminalCommand += 'kitty --start-as=normal --'
      break
    case 'alacritty':
      terminalCommand += 'alacritty -e'
      break
    case 'konsole':
      terminalCommand += 'konsole -e'
      break
  }
  templateInput.value = terminalCommand + " " + editorConfig.command + " " + templateTempFileName
}

async function saveSettings() {
  const editor = editorSelect.value
  const terminal = terminalSelect.value
  const shell = editor === 'custom' ? shellSelect.value : 'sh'
  const template = templateInput.value
  const bypassVersionCheck = bypassVersionCheckInput.checked
  await browser.storage.local.set({
    editor: editor,
    terminal: terminal,
    shell: shell,
    template: template,
    bypassVersionCheck: bypassVersionCheck,
  })
}

async function loadSettings() {
  const settings = await browser.storage.local.get(['editor', 'terminal', 'shell', 'template', 'bypassVersionCheck'])
  if (settings.editor) {
    editorSelect.value = settings.editor
    terminalSelect.value = settings.terminal
    shellSelect.value = settings.shell
    templateInput.value = settings.template
    bypassVersionCheckInput.checked = settings.bypassVersionCheck
    updateOptionsForEditor(settings.editor)
  } else {
    await updateTemplate()
  }
}

loadSettings()
