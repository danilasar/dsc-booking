function setLocation(curLoc){
    try {
        history.pushState(null, null, curLoc);
        return;
    } catch(e) {}
    location.hash = '#' + curLoc;
}

function loadAjaxNav() {
    let main = document.querySelector("body > main");
    let nav = document.getElementById("nav_main").querySelectorAll("a");
    nav.forEach((value, key, parent) => {
       value.addEventListener("click", async (event) => {
           event.preventDefault();
           let resp = await fetch(value.href, {
               headers: {
                   'X-Requested-With': 'XMLHttpRequest'
               }
           });
           let txt = await resp.text();
           main.innerHTML = txt;
           setLocation(value.href);
        })
    });
}

loadAjaxNav();