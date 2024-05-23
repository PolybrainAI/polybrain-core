/*

The Polybrain OnShape plugin is defined here

*/

const API_HOST = "http://localhost:30000"
const API_WS = "ws://localhost:30000"

var assistant;

if (typeof browser === "undefined") {
    var browser = chrome; // @ts-ignore
}


function extractDocumentId(url) {
    const regex = /documents\/([a-f0-9]{24})/;
    const match = url.match(regex);
    if (match) {
        return match[1];
    } else {
        console.debug(`Unable to extract document id from the url ${url}`)
        return null;
    }
}

function waitForElm(selector) {
    return new Promise(resolve => {
        if (document.querySelector(selector)) {
            return resolve(document.querySelector(selector));
        }

        const observer = new MutationObserver(mutations => {
            if (document.querySelector(selector)) {
                observer.disconnect();
                resolve(document.querySelector(selector));
            }
        });

        observer.observe(document.body, {
            childList: true,
            subtree: true
        });
    });
}

// Checks to see if the assistant should still be visible
let lastUrl = location.href; 
new MutationObserver(() => {
    const url = location.href;
    if (url !== lastUrl) {
        lastUrl = url;
        var state_in_modeler = !(extractDocumentId(window.location.href) == null);
        if (assistant != null && !state_in_modeler){
            assistant.remove()
            assistant=null;
        }
        else if (assistant == null && state_in_modeler){
            load_assistant()
        }
    }
}).observe(document, {subtree: true, childList: true});



async function swapMicrophoneEnabled() {
    // Switches image to microphone icon
    console.debug("Switching icon to microphone")
    image.style.transform = "scale(0)"

    await new Promise(r => setTimeout(r, 300)); // wait for animation to run

    image.src = browser.runtime.getURL('images/mic-fill.svg');
    image.style.transform = "scale(0.5)";
}

async function swapDefaultIcon() {
    // Switches image to default icon
    console.debug("Switching icon to default")
    image.style.transform = "scale(0)"

    await new Promise(r => setTimeout(r, 300)); // wait for animation to run

    image.src = browser.runtime.getURL('images/logo-nobackground.png');
    image.style.transform = "scale(1)";

    assistant.disabled = false
}


async function swapSpeakingIcon() {
    // Switches to the speaking icon
    console.debug("Switching icon to speaking")
    image.style.transform = "scale(0)"

    await new Promise(r => setTimeout(r, 300)); // wait for animation to run

    image.src = browser.runtime.getURL('images/soundwave.svg');
    image.style.transform = "scale(0.5)";
}

async function swapLoadingIcon() {
    // Switches to the loading icon
    console.debug("Switching icon to loading")
    image.style.transform = "scale(0)"

    await new Promise(r => setTimeout(r, 300)); // wait for animation to run

    image.src = browser.runtime.getURL('images/loading.svg');
    image.style.transform = "scale(1)";
}

async function handleMicrophoneToggle(data) {
    // Handles an MICROPHONE websocket message
    console.log("Dispatched as microphone toggle")
    if (data.body.STATUS == "ON") {
        swapMicrophoneEnabled()
    }
    else if (data.body.STATUS == "OFF") {
        swapDefaultIcon()
    }
    else {
        console.error("Data is illegal:")
        console.error(data)
    }
}
async function handleSpeakingToggle(data) {
    // Handles an SPEAKING websocket message
    console.log("Dispatched as speaking toggle")
    if (data.body.STATUS == "ON") {
        console.log("Starting to speak")
        swapSpeakingIcon()
    }
    else if (data.body.STATUS == "OFF") {
        console.log("Finished to speaking")
        swapDefaultIcon()
    }
    else {
        console.error("Data is illegal:")
        console.error(data)
    }
}

async function handleLoadingToggle(data) {
    // Handles an SPEAKING websocket message
    console.log("Dispatched as loading toggle")
    if (data.body.STATUS == "ON") {
        console.log("LLM loading...")
        swapLoadingIcon()
    }
    else if (data.body.STATUS == "OFF") {
        console.log("LLM done loading")
        swapDefaultIcon()
    }
}

var websocket;

async function startSession() {

    assistant.disabled = true

    const document_id = extractDocumentId(window.location.href);
    console.debug(`determined the document id to be ${document_id}`);

    var session_id;

    try {

        const response = await fetch(
            `${API_HOST}/session/create/${document_id}`,
        );
        console.log(response);
        session_id = (await response.json()).session_id

        console.log(`The session id is ${session_id}`)

    }
    catch {
        console.error("Failed to create a new session")
    }

    websocket = new WebSocket(`${API_WS}/ws/${session_id}`);

    websocket.addEventListener("open", (event) => {
        console.log(`Beginning new session ${session_id}`)
        websocket.send(JSON.stringify({ "messageType": "BEGIN", "body": {} }))
    });

    // Listen for messages
    websocket.addEventListener("message", (event) => {

        message = event.data.message

        message_data = JSON.parse(event.data)
        console.log(`Message from server:`, message_data)

        // Dispatch Message Types
        if (message_data["messageType"] == "MICROPHONE") {
            handleMicrophoneToggle(message_data)
        }
        if (message_data["messageType"] == "SPEAKING") {
            handleSpeakingToggle(message_data)
        }
        if (message_data["messageType"] == "LOADING") {
            handleLoadingToggle(message_data)
        }

    });

}

async function assistantClick() {
    console.log("assistant activated")
    await startSession()
}


// Load assistant
function load_assistant(event){

    if (extractDocumentId(window.location.href) != null){
        // Inject a assistant that can be clicked
        assistant = document.createElement('button');
        assistant.className = 'polybrain-assistant';
        document.body.appendChild(assistant);
        
        // Add image to the assistant
        var image = document.createElement('img')
        image.src = browser.runtime.getURL('images/logo-nobackground.png');
        assistant.appendChild(image)

        // Add click listener
        assistant.addEventListener('click', assistantClick);
    
        console.log("Polybrain loaded")
    }

}
window.addEventListener("load", load_assistant)