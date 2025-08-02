// Import the invoke function from the Tauri API
const { invoke } = window.__TAURI__.tauri;
const { WebviewWindow } = window.__TAURI__.window;
const { listen } = window.__TAURI__.event;

document.addEventListener("DOMContentLoaded", () => {
  const settingsBtn = document.getElementById("settings-btn");
  const settingsMenu = document.getElementById("settings-menu");
  const menuOverlay = document.getElementById("menu-overlay");
  const generateLinkBtn = document.getElementById("generate-link-btn");
  const setManualLinkBtn = document.getElementById("set-manual-link-btn");
  const refreshBtn = document.getElementById("refresh-btn");
  const vdoNinjaIframe = document.getElementById("vdo-ninja-iframe");

  // Toggle settings menu
  function toggleMenu() {
    const isOpen = settingsMenu.classList.contains("show");
    if (isOpen) {
      closeMenu();
    } else {
      openMenu();
    }
  }

  function openMenu() {
    settingsMenu.classList.add("show");
    menuOverlay.classList.add("show");
    settingsBtn.classList.add("active");
  }

  function closeMenu() {
    settingsMenu.classList.remove("show");
    menuOverlay.classList.remove("show");
    settingsBtn.classList.remove("active");
  }

  // Event listeners for menu
  settingsBtn.addEventListener("click", toggleMenu);
  menuOverlay.addEventListener("click", closeMenu);

  // Function to load the URL into the iframe
  const loadUrlIntoIframe = async () => {
    try {
      const url = await invoke("load_decrypted_url_from_config_command");
      if (url) {
        vdoNinjaIframe.src = url;
        console.log("Loaded URL into iframe:", url);
      } else {
        // If no URL is saved, generate a new one
        await invoke("generate_new_random_link");
        const newUrl = await invoke("load_decrypted_url_from_config_command");
        vdoNinjaIframe.src = newUrl;
        console.log("Generated and loaded new URL:", newUrl);
      }
    } catch (error) {
      console.error("Error loading URL:", error);
    }
  };

  // Listen for URL update events from Rust backend
  listen("url-updated", (event) => {
    console.log("URL updated event received:", event.payload);
    // Update iframe with the new URL immediately
    vdoNinjaIframe.src = event.payload;
  });

  // Event Listeners for menu buttons
  generateLinkBtn.addEventListener("click", async () => {
    closeMenu();
    await invoke("generate_new_random_link");
  });

  setManualLinkBtn.addEventListener("click", () => {
    closeMenu();
    const manualLinkWindow = new WebviewWindow("manual_link_dialog", {
      url: "manual_link.html",
      title: "Set Manual Link",
      width: 400,
      height: 350,
      resizable: false,
      center: true,
    });

    manualLinkWindow.once("tauri://webview-window-closed", () => {
      console.log("Manual link dialog closed");
    });
  });

  refreshBtn.addEventListener("click", () => {
    closeMenu();
    loadUrlIntoIframe();
  });

  // Close menu when clicking on menu items
  document.querySelectorAll(".menu-item").forEach((item) => {
    item.addEventListener("click", () => {
      setTimeout(() => closeMenu(), 100);
    });
  });

  // Initial load of the URL into the iframe
  loadUrlIntoIframe();
});
