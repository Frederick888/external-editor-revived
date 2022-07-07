const presetSelect = document.getElementById('preset')
const shellRow = document.getElementById('shell-row')
const shellSelect = document.getElementById('shell')
const templateInput = document.getElementById('template')
const bypassVersionCheckInput = document.getElementById('bypass-version-check')
presetSelect.onchange = (e) => {
  const preset = e.target.value;
  if (preset === 'custom') {
    shellRow.style = ''
    templateInput.removeAttribute('disabled')
  } else {
    shellRow.style = 'display: none;'
    templateInput.setAttribute('disabled', 'true')
  }
  updateTemplate()
}

const applyButton = document.getElementById('apply')
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

function updateTemplate() {
  const preset = presetSelect.value
  if (preset === 'custom') {
    return
  }
  let template
  switch (preset) {
    case 'konsole_nvim':
      template = 'konsole -e nvim -- "/path/to/temp.eml"'
      break;
    case 'konsole_vim':
      template = 'konsole -e vim -- "/path/to/temp.eml"'
      break;
    case 'kitty_nvim':
      template = 'kitty --start-as=normal -- nvim "/path/to/temp.eml"'
      break;
    case 'kitty_vim':
      template = 'kitty --start-as=normal -- vim "/path/to/temp.eml"'
      break;
    case 'neovide':
      template = 'neovide --nofork "/path/to/temp.eml"'
      break;
    default:
      template = 'konsole -e nvim -- "/path/to/temp.eml"'
      break;
  }
  templateInput.value = template
}

async function saveSettings() {
  const preset = presetSelect.value
  const shell = preset === 'custom' ? shellSelect.value : 'sh'
  const template = templateInput.value
  const bypassVersionCheck = bypassVersionCheckInput.checked
  await browser.storage.local.set({
    preset: preset,
    shell: shell,
    template: template,
    bypassVersionCheck: bypassVersionCheck,
  })
}

async function loadSettings() {
  const settings = await browser.storage.local.get(['preset', 'shell', 'template', 'bypassVersionCheck'])
  if (settings.preset) {
    presetSelect.value = settings.preset
    shellSelect.value = settings.shell
    templateInput.value = settings.template
    bypassVersionCheckInput.checked = settings.bypassVersionCheck
    if (settings.preset === 'custom') {
      shellRow.style = ''
      templateInput.removeAttribute('disabled')
    }
  } else {
    updateTemplate()
  }
}

loadSettings()
