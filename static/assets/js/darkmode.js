/* Utility functions for setting persistent dark mode */

function getDarkMode() {
    return Number(window.localStorage.getItem("DARK_MODE"));
}

function setDarkMode(value) {
    window.localStorage.setItem("DARK_MODE", Number(value));
}

function updateTheme() {
    var root = document.querySelector("body");
    if (getDarkMode()) {
        root.style.setProperty("--background-color-active", "var(--background-color-dark)");
        root.style.setProperty("--background-color-overlay-active", "var(--background-color-overlay-dark)");
        root.style.setProperty("--accent-color-active", "var(--accent-color-dark)");
    } else {
        root.style.setProperty("--background-color-active", "var(--background-color-light)");
        root.style.setProperty("--background-color-overlay-active", "var(--background-color-overlay-light)");
        root.style.setProperty("--accent-color-active", "var(--accent-color-light)");
    }
}

function toggleDarkMode() {
    setDarkMode(Number(!getDarkMode()));
    updateTheme();
}

document.addEventListener("DOMContentLoaded", function(event) {
    updateTheme();
});
