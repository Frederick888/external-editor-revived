const nativeAppName = "external_editor_revived"
const manifest = browser.runtime.getManifest()
const version = manifest.version
const port = browser.runtime.connectNative(nativeAppName)

const receivedPerTab = {}

async function commandListener(command) {
  console.debug(`${manifest.short_name} command: ${command}`)
  switch (command) {
    case 'create-with-send-on-exit':
      await browserActionListener(null, {
        modifiers: ['Shift']
      })
      break
    case 'compose-with-send-on-exit':
      await commandComposeWithSendOnExit()
      break
    case 'reply-to-sender':
    case 'reply-to-sender-with-send-on-exit':
      await commandReply('replyToSender', command.indexOf('send-on-exit') > 0)
      break
    case 'reply-to-list':
    case 'reply-to-list-with-send-on-exit':
      await commandReply('replyToList', command.indexOf('send-on-exit') > 0)
      break
    case 'reply-to-all':
    case 'reply-to-all-with-send-on-exit':
      await commandReply('replyToAll', command.indexOf('send-on-exit') > 0)
      break
  }
}

async function commandComposeWithSendOnExit() {
  const focusedTab = await getFocusedTab('messageCompose')
  if (!focusedTab) {
    createBasicNotification('command', `${manifest.short_name} shortcut error`, 'Failed to determine focused message compose tab')
    return
  }
  await composeActionListener(focusedTab, {
    modifiers: ['Shift']
  })
}

async function commandReply(replyType, sendOnExit) {
  let messages = null
  const currentMailTab = await messenger.mailTabs.getCurrent()
  if (currentMailTab) {
    const currentMailTabWindow = await browser.windows.get(currentMailTab.windowId)
    if (currentMailTabWindow.focused) {
      const selectedMessages = await messenger.mailTabs.getSelectedMessages(currentMailTab.id)
      if (!!selectedMessages) {
        messages = selectedMessages.messages
        console.debug(`${manifest.short_name} got selected messages from current mail tab: `, messages)
      }
    }
  }

  const currentMessageDisplayTab = await getFocusedTab('messageDisplay')
  if (currentMessageDisplayTab) {
    messages = await messenger.messageDisplay.getDisplayedMessages(currentMessageDisplayTab.id)
    console.debug(`${manifest.short_name} got messages from current message display: `, messages)
  }

  if (messages && messages.length > 0) {
    const tab = await messenger.compose.beginReply(messages[messages.length - 1].id, replyType)
    await composeActionListener(tab, {
      modifiers: sendOnExit ? ['Shift'] : []
    })
  }
}

async function browserActionListener(_tab, info) {
  const composeTab = await messenger.compose.beginNew()
  await composeActionListener(composeTab, info)
}

async function composeActionListener(tab, info) {
  if (!await messenger.composeAction.isEnabled({tabId: tab.id})) {
    return
  }
  const settings = await browser.storage.local.get(['editor', 'shell', 'template', 'suppressHelpHeaders', 'bypassVersionCheck'])
  if (!settings.editor) {
    await createBasicNotification(
      'no-settings',
      `${manifest.short_name} needs to be configured first!`,
      `Please go to Add-ons and Themes -> Extensions -> ${manifest.short_name} to configure this extension.`
    )
    return
  }
  const composeDetails = await messenger.compose.getComposeDetails(tab.id)
  const attachments = await messenger.compose.listAttachments(tab.id)
  composeDetails.attachments = JSON.parse(JSON.stringify(attachments))
  const request = {
    configuration: {
      version: manifest.version,
      shell: settings.shell,
      template: settings.template,
      sendOnExit: info.modifiers.indexOf('Shift') >= 0,
      suppressHelpHeaders: !!settings.suppressHelpHeaders,
      bypassVersionCheck: !!settings.bypassVersionCheck,
    },
    tab: tab,
    composeDetails: composeDetails,
  }
  console.debug(`${manifest.short_name} sending: `, request)
  try {
    port.postMessage(toPlainObject(request))
    await messenger.composeAction.disable(tab.id)
  } catch (_) {
    await createBasicNotification('port', `${manifest.short_name} failed to talk to messaging host`, 'Please check Tools -> Developer Tools -> Error Console for details')
  }
}

async function nativeMessagingListener(response) {
  console.debug(`${manifest.short_name} received: `, response)
  if (response.title && response.message) {
    await createBasicNotification('', response.title, response.message)
  } else {
    response.composeDetails.attachments = []

    if (receivedPerTab[response.tab.id] === undefined) {
      receivedPerTab[response.tab.id] = []
    }
    receivedPerTab[response.tab.id].push(response)
    if (receivedPerTab[response.tab.id].length == response.configuration.total) {
      await messenger.composeAction.enable(response.tab.id)
      receivedPerTab[response.tab.id].sort((a, b) => a.configuration.sequence - b.configuration.sequence)
      const composeDetails = receivedPerTab[response.tab.id][0].composeDetails
      for (let i = 1; i < receivedPerTab[response.tab.id].length; i++) {
        if (typeof composeDetails.plainTextBody === 'string') {
          composeDetails.plainTextBody += receivedPerTab[response.tab.id][i].composeDetails.plainTextBody
        }
        if (typeof composeDetails.body === 'string') {
          composeDetails.body += receivedPerTab[response.tab.id][i].composeDetails.body
        }
      }
      if (!!response.warnings) {
        for (const warning of response.warnings) {
          await createBasicNotification('warning', warning.title, warning.message)
        }
      }
      await messenger.compose.setComposeDetails(response.tab.id, composeDetails)
      if (response.configuration.sendOnExit) {
        try {
          await messenger.compose.sendMessage(response.tab.id)
        } catch (_) {
          // only catchable on Thunderbird >= 102
          createBasicNotification('send', `${manifest.short_name} failed to send message`, 'Please check if you have fill in recipients and other mandatory fields')
        }
      }
      delete receivedPerTab[response.tab.id]
    }
  }
}

async function nativeMessagingDisconnectListener(p) {
  let message = 'Please try restarting Thunderbird'
  if (p.error) {
    message = `${p.error.message}. Please try restarting Thunderbird`
  }
  await createBasicNotification('port', `${manifest.short_name} messaging host disconnected`, message)
}

function toPlainObject(o) {
  // https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Chrome_incompatibilities#data_cloning_algorithm
  // Extension that rely on the toJSON() method of the JSON serialization
  // algorithm can use JSON.stringify() followed by JSON.parse() to ensure that
  // a message can be exchanged, because a parsed JSON value is always
  // structurally cloneable.
  return JSON.parse(JSON.stringify(o))
}

async function getFocusedTab(tabType) {
  const windows = await browser.windows.getAll({})
  const focusedWindows = windows.filter((w) => w.focused)
  if (focusedWindows.length != 1) {
    console.debug(`${manifest.short_name} got ${tabType} windows: `, windows)
    return null
  }
  const focusedWindow = focusedWindows[0]

  const tabs = await browser.tabs.query({
    active: true,
    type: tabType,
  })
  const focusedTabs = tabs.filter((t) => t.windowId === focusedWindow.id)
  if (focusedTabs.length != 1) {
    console.debug(`${manifest.short_name} got ${tabType} tabs: `, tabs)
    return null
  }

  return tabs[0]
}

async function createBasicNotification(id, title, message, eventTime = 5000) {
  await browser.notifications.create(id, {
    type: 'basic',
    title: title,
    message: message,
    eventTime: eventTime,
  })
}

messenger.commands.onCommand.addListener(commandListener)
messenger.browserAction.onClicked.addListener(browserActionListener)
messenger.composeAction.onClicked.addListener(composeActionListener)
port.onMessage.addListener(nativeMessagingListener)
port.onDisconnect.addListener(nativeMessagingDisconnectListener)
