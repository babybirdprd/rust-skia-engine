# Feature Matrix & Creative Workflows

## 1. The Core Integration Matrix
How our four pillars combine to create unique features.

| **Capability** | **SAM 3 (Vision)** | **Parakeet (Hearing)** | **TTS (Speech)** | **Director Engine (Action)** |
| :--- | :--- | :--- | :--- | :--- |
| **"Smart Captions"** | Detects face position to avoid covering it. | Generates word-level timestamps. | N/A | Renders animated text at safe coordinates using Taffy layouts. |
| **"The Narrator"** | Identifies objects ("That looks like a vintage car"). | N/A | Generates voiceover: "Check out this vintage ride." | Assembles B-roll of cars, syncing cuts to the voiceover audio peaks. |
| **"Stock Remix"** | Segments foreground actors from generic stock footage. | N/A | N/A | Places actors into new 3D/2D environments (e.g., "Office worker in space"). |
| **"Localization"** | Tracks lip movement (future). | Transcribes original audio. | Generates translated audio in same voice tone. | Re-times video playback speed to match new audio duration (elastic time). |

## 2. Flagship Workflows (The "Recipes")

### Recipe A: The "Stock Media Alchemy" (Asset Repurposing)
* **Problem:** Stock footage is boring and generic.
* **The Director Fix:**
    1.  **Ingest:** Load a folder of 10 generic "Corporate Meeting" stock clips.
    2.  **Segment:** SAM 3 automatically extracts the "People" from the "Boardroom Backgrounds."
    3.  **Re-Contextualize:** The Engine places these people over a new background (e.g., a futuristic UI or a branded color solid).
    4.  **Result:** Unique, branded assets created from generic stock, without manual rotoscoping.

### Recipe B: The "Reactive" Social Clip
* **Problem:** Vertical video cropping usually cuts off the action.
* **The Director Fix:**
    1.  **Track:** SAM 3 identifies the "Subject" (e.g., the skateboarder).
    2.  **Listen:** Parakeet identifies the word "Jump!" in the audio.
    3.  **Effect:** The Engine applies a "Camera Shake" effect and a "Zoom" specifically at the timestamp where "Jump!" occurs, while keeping the skateboarder perfectly centered in the 9:16 frame.

### Recipe C: The "Invisible" Localizer
* **Problem:** Dubbing videos ruins the pacing.
* **The Director Fix:**
    1.  **Transcribe:** Parakeet gets English text.
    2.  **Translate:** LLM translates to Spanish.
    3.  **Speak:** TTS generates Spanish audio.
    4.  **Re-Time:** The Engine calculates that Spanish is 20% longer. It automatically slows down the B-roll clips (using Optical Flow interpolation if needed) to perfectly match the new audio track length.

---

## 3. Future Roadmap & Planned Features

### Phase 1: The Essentials (MVP)
* [ ] **Object Pinning:** `node.pin_to_object("face")`.
* [ ] **Auto-Subtitles:** `scene.add_captions(style: "karaoke")`.
* [ ] **Smart Crop:** `video.auto_reframe(aspect: "9:16", target: "main_actor")`.

### Phase 2: The "Magician" Tools
* [ ] **Object Removal:** "Delete the trash can." (Inpainting).
* [ ] **Depth Sorting:** "Put text behind the person." (Z-Index manipulation).
* [ ] **Style Transfer:** "Make the sky look like a painting." (Mask-constrained shaders).

### Phase 3: The "Generative" Director
* [ ] **Script-to-Video:** User types a script -> LLM generates Rhai -> Engine gathers stock -> SAM 3 refines assets -> Final render.