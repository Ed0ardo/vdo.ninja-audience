// Import necessary functions from the Tauri API
const { invoke } = window.__TAURI__.tauri;
const { appWindow } = window.__TAURI__.window;

// Password visibility toggle
const togglePasswordBtn = document.getElementById("toggle-password-btn");
const audienceInput = document.getElementById("audience");

togglePasswordBtn.addEventListener("click", () => {
  if (audienceInput.type === "password") {
    audienceInput.type = "text";
    togglePasswordBtn.textContent = "🙈";
  } else {
    audienceInput.type = "password";
    togglePasswordBtn.textContent = "👁️";
  }
});

// --- DOM Element References ---
document.getElementById("save-btn").addEventListener("click", async () => {
  const pushId = document.getElementById("push_id").value;
  const audience = document.getElementById("audience").value;
  const errorMessageDiv = document.getElementById("error-message");
  errorMessageDiv.textContent = ""; // Clear previous errors

  // --- Input Validation ---
  if (pushId) {
    try {
      // Invoke the Rust command to set the manual link
      await invoke("set_and_save_vdo_ninja_link", { pushId, audience });
      // Close the dialog window on success.
      await appWindow.close();
    } catch (error) {
      // --- Error Handling ---
      // Display any validation errors returned from the Rust backend.
      errorMessageDiv.textContent = error;
    }
  } else {
    // --- Frontend Validation ---
    // Basic check to ensure pushId is not empty.
    errorMessageDiv.textContent = "Push ID (Room Name) is required.";
  }
});

// --- Cancel Button Functionality ---
document.getElementById("cancel-btn").addEventListener("click", async () => {
  // Close the dialog window without saving.
  await appWindow.close();
});
