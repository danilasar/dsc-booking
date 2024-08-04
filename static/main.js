function setLocation(curLoc){
    try {
        history.pushState(null, null, curLoc);
        return;
    } catch(e) {}
    location.hash = '#' + curLoc;
}
function loadAjaxNav() {
    let default_main = document.querySelector("body > #wrap > main");
    let nav = document.querySelectorAll("a[data-ajax]");
    nav.forEach((value, key, parent) => {
       value.addEventListener("click", async (event) => {
           event.preventDefault();
           let resp = await fetch(value.href, {
               headers: {
                   'X-Requested-With': 'XMLHttpRequest'
               }
           });
           let txt = await resp.text();
           let customSelector = value.attributes["data-ajax"];
           if(typeof customSelector === "string" && customSelector !== "") {
               document.querySelector(customSelector).innerHTML = txt;
           } else {
               default_main.innerHTML = txt;
           }
           setLocation(value.href);
        })
    });
}

loadAjaxNav();