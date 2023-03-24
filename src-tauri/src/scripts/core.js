// *** Core Script - IPC ***

async function init() {
    console.log("app init ...")
    let nav = document.getElementsByClassName('bp3-navbar')
    nav[0].style['display'] = 'none'
    // disable context menu
    document.addEventListener('contextmenu', event => event.preventDefault());
}

if (
    document.readyState === "complete" ||
    document.readyState === "interactive"
  ) {
    init();
  } else {
    document.addEventListener("DOMContentLoaded", init);
  }