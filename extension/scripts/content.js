/*

The Polybrain OnShape plugin is defined here

*/


if (typeof browser === "undefined") {
    var browser = chrome;
}


// Inject a circle that can be clicked
var circle = document.createElement('button');
circle.className = 'polybrain-assistant';
document.body.appendChild(circle);

// Add image to the circle
image = document.createElement('img')
image.src = browser.runtime.getURL('images/logo-nobackground.png');
circle.appendChild(image)

function swapToMicrophone() {
    // Switches image to microphone icon
    console.debug("Switching icon to microphone")
    image.style.transform = "scale(0)"
    
    setTimeout(() => {
        image.src = browser.runtime.getURL('images/mic-fill.svg');
        image.style.transform = "scale(0.5)";
    }, 300)
    
    setTimeout(() => { swapToDefault() }, 5000) // This will eventually be triggered
}

function swapToDefault() {
    // Switches image to default icon
    console.debug("Switching icon to default")
    image.style.transform = "scale(0)"

    setTimeout(() => {
        image.src = browser.runtime.getURL('images/logo-nobackground.png');
        image.style.transform = "scale(1)";
    }, 300)
    circle.disabled = false
}



function assistantClick() {
    console.log("assistant activated")
    circle.disabled = true
    swapToMicrophone()
}




// Add click listener
circle.addEventListener('click', assistantClick);
