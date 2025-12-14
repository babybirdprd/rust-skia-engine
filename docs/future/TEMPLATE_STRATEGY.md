# Template & Abstraction Architecture

## 1\. The Philosophy: "Behaviors, not Boilerplate"

In `Director`, a template is not just a `.dirproj` file with a placeholder image. A template is a **reusable Rhai module** that defines a specific *behavior* or *style*.

We classify templates into three distinct layers of abstraction:

## Layer 1: The "Macro" (Functional Templates)

  * **Definition:** Small, reusable snippets of logic that abstract away math or engine commands. These are effectively "Functions" for the end-user.
  * **Storage:** `assets/scripts/std/animation.rhai`
  * **Examples:**
      * `fly_in_up(duration)`: Handles opacity 0-\>1 and translate\_y 50-\>0.
      * `pulse_on_beat(bpm)`: Scales the node up/down in sync with a tempo.
      * `typewriter_effect(speed)`: Reveals text character-by-character.
  * **Why it matters:** This allows the "Local AI" (or a human) to write concise scripts (`title.typewriter_effect(0.5)`) instead of hallucinating raw loop logic.

## Layer 2: The "Smart Component" (Structural Templates)

  * **Definition:** A pre-configured `Composition` (Node Tree) that has built-in internal logic. It isn't just a layout; it reacts to its content.
  * **Storage:** `assets/components/smart_lower_third.dirproj`
  * **Examples:**
      * **"Smart Lower Third":** A text box that automatically resizes based on the length of the name *and* ensures it doesn't cover the face (using SAM 3 data passed from the parent).
      * **"Auto-B-Roll Frame":** A picture-in-picture frame that slowly zooms in ("Ken Burns effect") and automatically cuts to the next image when the audio hits a silence gap.
  * **The "Interface":** These components expose specific "Props" to the main timeline (e.g., `Color`, `Name`, `Position`), hiding the complex node graph inside.

## Layer 3: The "Recipe" (Project Templates)

  * **Definition:** A full-scale orchestration script that defines the **Arc** of a video. It takes raw assets as input and produces a finished video.
  * **Storage:** `assets/recipes/tiktok_explainer.rhai`
  * **Examples:**
      * **"The Faceless Historian":**
        1.  Input: Script text + folder of images.
        2.  Action: Generates Voiceover (Supertonic).
        3.  Action: Aligns images to sentences.
        4.  Action: Applies "Old Film" effect to images.
        5.  Action: Adds captions.
      * **"The Podcast Clipper":**
        1.  Input: 60min video.
        2.  Action: Finds highest audio volume moments.
        3.  Action: Cuts 30s clip.
        4.  Action: Auto-reframes to 9:16 keeping active speaker centered.

-----

## 2\. The "Adaptive" Strategy (Cloud vs. Local)

You raised a critical point: *What if the user has a weak computer?*

Our templates must be **Capability-Aware**.
In the Rhai environment, we expose a capability flag: `engine.capabilities.has_gpu`.

**Example: The "Background Removal" Template**

```rust
// Inside `assets/scripts/std/effects.rhai`

fn remove_background(video_node) {
    if engine.capabilities.has_gpu && engine.capabilities.sam3_loaded {
        // High-End: Use SAM 3 for perfect segmentation
        let mask = video.track_object("person");
        video_node.set_mask(mask);
    } else {
        // Low-End / Fallback: Use a simple center crop or vignette
        // Graceful degradation so the video can still be rendered
        print("Warning: GPU not found. Falling back to simple mask.");
        let circle_mask = create_circle_mask();
        video_node.set_mask(circle_mask);
    }
}
```

**Benefit:** You build *one* template. It works on an RTX 4090 (Cinema Quality) and a MacBook Air (Draft Quality) automatically.

## 3\. Implementation: The "Asset Store"

Since these templates are just files (Rhai scripts or `.dirproj` JSONs), they are highly portable.

  * **Community Library:** Users can share "Recipes" (e.g., "Here is my script for 'VSauce-style' pacing").
  * **Official Packs:** You sell curated, high-quality "Smart Components" (e.g., "The Youtuber Essentials Pack" - includes Animated Subtitles, Like/Subscribe buttons that auto-color match).