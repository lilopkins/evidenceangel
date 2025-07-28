let tabAnchorMatch = () => {
    let tabRegex = /^#tab(\d+)$/;
    let pageHash = window.location.hash;

    if (pageHash) {
        let match = pageHash.match(tabRegex);
        if (match && match[1]) {
            // Found tab index
            let tabIndex = parseInt(match[1]);
            // Hide all tabs and remove selected from tab links
            document
                .querySelectorAll("[data-tab-index]")
                .forEach((e) => e.classList.remove("selected"));
            // Show tab by index
            document
                .querySelectorAll(`[data-tab-index='${tabIndex}']`)
                .forEach((e) => e.classList.add("selected"));
        }
    }
};

window.addEventListener('DOMContentLoaded', () => {
    const showAll = document.getElementById('showAll');
    const body = document.querySelector('body');
    const updateShowAll = () => {
        if (showAll.checked) {
            body.classList.add('show-all');
        } else {
            body.classList.remove('show-all');
        }
    };
    showAll.addEventListener('click', () => {
        updateShowAll();
    });
    // And in case the page has some reloaded state
    updateShowAll();

    hljs.highlightAll();
});

window.addEventListener("hashchange", tabAnchorMatch);
window.addEventListener("DOMContentLoaded", tabAnchorMatch);
