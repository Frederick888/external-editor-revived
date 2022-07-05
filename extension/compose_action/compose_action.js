async function load() {
  const bodyParagraph = document.getElementById('main')
  bodyParagraph.textContent = 'Initialised!'
  messenger.composeAction.onClicked.addListener(composeActionListener)
}

async function composeActionListener(tab, info) {
  const composeDetails = await messenger.compose.getComposeDetails(tab)
  const bodyParagraph = document.getElementById('main')
  bodyParagraph.textContent = composeDetails.body
}

document.addEventListener('DOMContentLoaded', load)
