# User Requirement
now listen to my requirement: I want to do everything free, like using free API keys for fixing the transcription and using another free API key for voice to text. I want to add this in my current project.

---

# Project: ZyloFlow — AI-Powered Voice-to-Text Desktop Application

## 1. Project Overview

Build ZyloFlow, a lightweight, premium-looking, AI-powered voice-to-text desktop application inspired by Wispr Flow.

The application allows users to dictate text into any focused text input across their Linux desktop using configurable global keyboard shortcuts.

The user should be able to:
- Start voice dictation using a global hotkey.
- Record audio through the system microphone.
- Convert speech into text using Whisper-based transcription APIs.
- Refine and clean the transcription using an LLM.
- Automatically insert the refined text into the currently focused input field.
- Configure their API keys, AI providers, models, and keyboard shortcuts.
- Run ZyloFlow in the background without opening the main window every time.

## 2. Technology Stack

### Frontend
- React
- TypeScript
- Vite
- Tailwind CSS
- shadcn/ui
- Lucide React
- Zustand for application state management

### Desktop Backend
- Tauri v2
- Rust
- Tauri Global Shortcut Plugin
- Tauri Autostart Plugin
- Tauri System Tray Plugin
- Tauri Clipboard Manager Plugin
- Rust-based OS integration

### Audio
- Web Audio API
- MediaDevices API
- MediaRecorder API
- Microphone permission management
- Audio recording in the background

### AI Providers

Implement a modular AI provider architecture that allows providers and models to be changed through the application settings.

#### Speech-to-Text Providers

**Provider 1: Groq Whisper API**

API Base URL:
https://api.groq.com/openai/v1

Transcription Endpoint:
POST /audio/transcriptions

Recommended model:
whisper-large-v3-turbo

Alternative model:
whisper-large-v3

API key:
GROQ_API_KEY

Official documentation:
https://console.groq.com/docs/speech-to-text

**Provider 2: OpenAI Whisper API**

API Base URL:
https://api.openai.com/v1

Transcription Endpoint:
POST /audio/transcriptions

Recommended model:
whisper-1

API key:
OPENAI_API_KEY

Official documentation:
https://platform.openai.com/docs/guides/speech-to-text

#### Text Refinement Providers

**Provider 1: Google Gemini**

API:
Google Gemini Developer API

Recommended model:
gemini-2.5-flash

API key:
GEMINI_API_KEY

Official documentation:
https://ai.google.dev/gemini-api/docs

**Provider 2: Groq LLM**

API Base URL:
https://api.groq.com/openai/v1

Endpoint:
POST /chat/completions

Recommended models:
- openai/gpt-oss-120b
- openai/gpt-oss-20b

API key:
GROQ_API_KEY

Official documentation:
https://console.groq.com/docs/text-chat

**Provider 3: xAI Grok**

API Base URL:
https://api.x.ai/v1

Endpoint:
POST /chat/completions

Recommended model:
grok-4.1-fast

API key:
XAI_API_KEY

Official documentation:
https://docs.x.ai/

**Provider 4: OpenAI**

API Base URL:
https://api.openai.com/v1

Endpoint:
POST /chat/completions

Recommended model:
gpt-4.1-mini

API key:
OPENAI_API_KEY

Official documentation:
https://platform.openai.com/docs/api-reference/chat

Important:
- Keep all model IDs configurable.
- Do not hardcode model IDs into business logic.
- Verify current model availability and provider API compatibility before implementation.
- Allow users to select different providers for transcription and refinement.
- Reuse a single provider key when the user selects the same provider for both tasks.
- Never expose API keys in frontend logs, console output, or error messages.

## 3. Application UI

Create a simple, premium, minimal desktop interface.

Avoid unnecessary animations, gradients, excessive cards, and complex navigation.

### Main Dashboard

Display:
- ZyloFlow logo and application name.
- Application status: Active / Paused.
- Microphone status.
- Selected transcription provider.
- Selected refinement provider.
- Current global dictation shortcut.
- Start Dictation button.
- Stop Dictation button.
- Settings button.
- Recent transcription history.

The dashboard should remain lightweight and responsive.

### Settings Page

Create the following settings sections:

#### AI Providers
- Select speech-to-text provider.
- Select text refinement provider.
- Select transcription model.
- Select refinement model.
- Enter API keys.
- Show / hide API keys.
- Test API key connection.
- Display provider connection status.
- Display API errors in a human-readable format.

#### Hotkeys

Provide three configurable global shortcuts:

1. Toggle Dictation
   - Default: Ctrl + Alt + Space
   - Starts recording when inactive.
   - Stops recording when active.

2. Start Application
   - Default: Ctrl + Alt + S
   - Activates the application if it has been paused.
   - Does not open the main window.

3. Stop Application
   - Default: Ctrl + Alt + X
   - Stops dictation and disables application activity.
   - Does not terminate the process unless explicitly configured.

Allow the user to:
- Record a new shortcut using a keyboard capture interface.
- Reset shortcuts to defaults.
- Check shortcut registration status.
- Detect duplicate shortcuts within ZyloFlow.
- Handle shortcuts that conflict with operating-system or other application bindings.
- Display a clear error if a shortcut cannot be registered.

Do not falsely claim to detect every shortcut used by other applications. Use native registration results and desktop-environment-specific conflict checks where available.

#### General Settings
- Launch ZyloFlow at system startup.
- Start minimized to system tray.
- Automatically activate on login.
- Enable / disable background mode.
- Enable / disable text refinement.
- Enable / disable transcription history.
- Select audio language.
- Configure clipboard fallback.
- Configure notification behavior.

## 4. Global Hotkey Behavior

Global shortcuts must work outside the ZyloFlow window.

Examples:
- Browser input fields.
- VS Code.
- Terminal applications.
- Text editors.
- Chat applications.
- Other desktop applications.

Use the Tauri Global Shortcut Plugin with Rust integration.

The shortcut event must be handled by the native backend rather than depending on a React window being focused.

The main application window must not open when a shortcut is triggered.

## 5. Dictation Workflow

Implement the following complete workflow:

### Step 1: Activate Dictation

When the user presses the configured Toggle Dictation shortcut:

- Detect the global shortcut event in Rust.
- Verify the application is active.
- Verify that API keys are configured.
- Begin microphone recording.
- Show a small floating popup.
- Display the current state: "Listening..."
- Do not open the dashboard.

### Step 2: Audio Recording

Use the Web Audio API and MediaRecorder API to capture microphone audio.

Requirements:
- Capture audio with suitable sample rate and format.
- Handle microphone permission errors.
- Handle microphone device disconnection.
- Handle empty audio recordings.
- Handle recording cancellation.
- Handle maximum recording duration.
- Release microphone resources after recording.

Important:
Do not assume the Web Audio API automatically supports system-wide microphone recording when the React window is hidden. Implement and test background audio capture under Tauri on Linux.

### Step 3: Speech-to-Text

Send the recorded audio to the configured transcription provider.

Use the appropriate multipart/form-data request.

For Groq:
POST https://api.groq.com/openai/v1/audio/transcriptions

For OpenAI:
POST https://api.openai.com/v1/audio/transcriptions

The backend should:
- Handle network errors.
- Handle API rate limits.
- Handle invalid API keys.
- Handle transcription timeouts.
- Handle unsupported audio formats.
- Return the transcription text to the refinement pipeline.

### Step 4: AI Text Refinement

Send the raw transcription to the configured LLM.

Use the following system prompt:

"You are an intelligent voice transcription refinement engine.

Your task is to clean up raw speech transcription.

Rules:
- Correct spelling and grammatical errors.
- Add appropriate punctuation.
- Remove unnecessary filler words when appropriate.
- Improve sentence structure without changing meaning.
- Preserve technical terms, code, names, and product names.
- Preserve the original language.
- Do not translate unless explicitly requested.
- Do not add new information.
- Do not answer questions contained in the transcription.
- Do not summarize.
- Do not explain your corrections.
- Return only the refined text."

Provide the user with an optional refinement mode:
- Minimal correction.
- Standard correction.
- Professional correction.

### Step 5: Insert Text

After refinement:

- Insert the text into the currently focused input field.
- Preserve the user's existing content.
- Avoid replacing selected content unintentionally.
- Prefer reliable native text insertion.
- Use clipboard paste as a fallback.
- Restore the previous clipboard contents where possible.
- Avoid exposing clipboard contents in logs.
- Handle input fields that do not accept clipboard insertion.

Do not assume that a browser-based clipboard API can insert text into applications outside the browser.

### Step 6: Finish

- Stop recording.
- Release microphone resources.
- Close the floating popup.
- Return ZyloFlow to its idle state.
- Save the transcription only if history is enabled.

## 6. Floating Popup

Create a small floating popup that does not open the main application.

Popup states:

1. Idle
2. Listening
3. Transcribing
4. Refining
5. Inserting text
6. Completed
7. Error

Example:

Listening...
[ Microphone icon ] [ Recording animation ]

Transcribing...
Converting your speech to text

Refining...
Cleaning up your text

Completed
Text inserted successfully

The popup must:
- Remain small and unobtrusive.
- Use a transparent or minimal background.
- Stay above other windows.
- Avoid stealing keyboard focus.
- Be positioned near the top-center or bottom-center of the screen.
- Automatically disappear after completion.
- Support cancellation.
- Never open the full application window during shortcut execution.

## 7. Background Mode and Startup

ZyloFlow must run as a background desktop application.

On the first installation:
- Complete onboarding.
- Ask for microphone permissions.
- Ask the user to configure at least one API key.
- Allow the user to configure their hotkeys.
- Enable launch-at-login after explicit user consent.
- Start the application in background mode.

After a system reboot:
- ZyloFlow should automatically start if launch-at-login is enabled.
- Global shortcuts should be registered.
- The application should remain available without requiring the dashboard to be opened.

Use:
- Tauri Autostart Plugin.
- Tauri System Tray Plugin.
- Native Rust background lifecycle management.

The application should not silently enable startup behavior without informing the user.

### Tray Menu

Add a system tray menu:
- Open ZyloFlow
- Start Dictation
- Pause Application
- Resume Application
- Settings
- Quit

Quit should completely unregister shortcuts and release microphone resources.

## 8. Linux OS Integration

Target Ubuntu Linux first.

Build for the user's Ubuntu distribution.

Requirements:
- Tauri desktop application.
- Rust backend.
- Native global shortcut integration.
- Startup integration.
- System tray integration.
- Floating popup.
- Text insertion into other applications.
- Microphone permissions.
- Application packaging.

Important Linux constraints:

- Test X11 and Wayland separately.
- Global shortcuts and text insertion can behave differently across desktop environments.
- Wayland may restrict global hotkey registration and cross-application input injection.
- Do not assume X11-specific commands will work under Wayland.
- Provide graceful fallback behavior.
- Detect the active display server.
- Document any unsupported functionality.

Do not use a browser-only implementation for global hotkeys.

## 9. API Key Security

Store API keys securely.

Requirements:
- Prefer OS-native secure storage through a suitable Rust keyring integration.
- Never hardcode API keys.
- Never store API keys in plaintext configuration files.
- Never expose keys to browser DevTools.
- Avoid transmitting API keys through unnecessary frontend commands.
- Redact API keys from logs.
- Allow users to remove or rotate keys.

API calls should preferably be made from the Rust backend.

## 10. Architecture

Use a modular architecture.

Suggested Rust modules:

src-tauri/src/
- main.rs
- lib.rs
- hotkeys.rs
- audio.rs
- transcription.rs
- refinement.rs
- text_injection.rs
- secure_storage.rs
- tray.rs
- popup.rs
- settings.rs
- lifecycle.rs
- error.rs

Suggested React structure:

src/
- app/
- components/
- pages/
- hooks/
- stores/
- services/
- types/
- utils/

Use a provider abstraction:

TranscriptionProvider
- GroqWhisperProvider
- OpenAIWhisperProvider

RefinementProvider
- GeminiProvider
- GroqProvider
- XAIProvider
- OpenAIProvider

Keep API logic, UI logic, native OS integration, and state management separated.

## 11. Implementation Requirements

Do not build only a static UI.

Implement actual functionality.

Requirements:
- All buttons must work.
- All settings must persist.
- Hotkeys must be registered natively.
- Audio must be recorded.
- APIs must be integrated.
- Text must be refined.
- Text must be inserted into external applications.
- Background mode must work.
- System tray must work.
- Startup behavior must work.

If a feature cannot be implemented reliably on a particular Linux desktop environment, explain the limitation and implement the best supported alternative.

## 12. Testing

Test:
- First-time onboarding.
- API key validation.
- Hotkey registration.
- Hotkey conflicts.
- Microphone access.
- Audio recording.
- Transcription.
- Refinement.
- Text insertion.
- Background operation.
- Application startup after reboot.
- System tray functionality.
- Error handling.
- Network failure handling.
- X11 functionality.
- Wayland functionality where supported.

Do not claim functionality is tested if it has not been executed.

## 13. Build and Deliverables

Target:
Ubuntu Linux x86_64.

Deliver:
- Complete React frontend.
- Complete Rust backend.
- Tauri configuration.
- API provider integrations.
- Working hotkey system.
- Background lifecycle implementation.
- Secure API key storage.
- System tray.
- Floating popup.
- README.
- Installation instructions.
- Build instructions.
- Troubleshooting guide.

Compile the application.

Generate:
- .deb package
- AppImage if supported by the build environment

Verify the build artifacts exist.

Provide the exact installation and execution commands.

Do not stop after generating the source code. Build the application and resolve compilation errors.
>>>>>>> a40934d8418299c48f8d2ab6a4392f2321731f03
