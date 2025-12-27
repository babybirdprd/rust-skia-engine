# SAM 3 Integration Specification

## 1\. Core Capabilities

We utilize SAM 3 for three distinct tasks.

### A. "The Pin" (Point Tracking)

  * **Input:** `(x, y)` coordinate on Frame 0.
  * **Output:** A stream of `(x, y)` coordinates for Frame 1..N.
  * **Use Case:**
      * Attaching a text label to a moving car.
      * Stabilizing a shaky camera on a specific object.

### B. "The Matte" (Video Segmentation)

  * **Input:** A "Prompt" (Click or Box) on Frame 0.
  * **Output:** A binary Alpha Mask (Bitmap) for every frame.
  * **Use Case:**
      * **Rotoscoping:** Separating a person from the background to put text behind them.
      * **Color Grading:** Brightening *only* the face.
      * **Removal:** Inpainting out a specific object.

### C. "The Scout" (Open Vocabulary Detection - SAM 3 Exclusive)

  * **Input:** Text String ("red hat", "cat").
  * **Output:** Bounding Boxes `Rect` for all matching objects.
  * **Use Case:**
      * **Auto-B-Roll:** "Find the clip where a 'Dog' is visible."
      * **Smart Crop:** "Ensure 'The Speaker' is always in the center 9:16 frame."

## 2\. Technical Constraint Strategy

SAM 3 is heavy. We cannot run it blindly on every frame for every object.

### The "Bake" Workflow

1.  **Draft Mode:** User/Script requests tracking. We run SAM 3 on *key frames only* (e.g., every 10th frame) and interpolate linearly. This is fast enough for UI preview.
2.  **Render Mode:** When exporting, we run inference on *every* frame for pixel-perfect masks.

### The "Caching" Layer

  * **Hash:** `MD5(VideoID + PromptType + PromptValue)`
  * **Storage:** We save the generated masks to disk (`cache/masks/`).
  * **Why:** If the user changes the text color, we do NOT re-run SAM 3. We just re-composite the cached mask. This makes iteration instant.

## 3\. Future "Advanced" Use Cases

  * **3D Space Inference:** Using SAM 3 masks + Monocular Depth (like Depth-Anything) to create a "2.5D" scene, allowing true 3D text insertion.
  * **Object Removal (Inpainting):** Using the SAM 3 mask as the "hole" for a LaMa inpainting model to fill.