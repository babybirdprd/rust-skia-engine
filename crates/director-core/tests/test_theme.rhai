
// Test Spacing
let s = theme::space("md");
if s != 16.0 {
    print("Error: theme::space('md') expected 16.0, got " + s);
    exit;
}
let s_default = theme::space("unknown");
if s_default != 16.0 {
    print("Error: theme::space('unknown') expected 16.0, got " + s_default);
    exit;
}

// Test Safe Area
let sa = theme::safe_area("tiktok");
if sa.top != 100.0 {
    print("Error: theme::safe_area('tiktok').top expected 100.0, got " + sa.top);
    exit;
}
let sa_default = theme::safe_area("unknown_platform");
if sa_default.top != 64.0 { // desktop default
    print("Error: theme::safe_area('unknown_platform').top expected 64.0 (desktop), got " + sa_default.top);
    exit;
}
if sa.contains("aspect_ratio") {
    print("Error: theme::safe_area should not contain 'aspect_ratio'");
    exit;
}

// Test Border Radius
let r = theme::radius("lg");
if r != 12.0 {
    print("Error: theme::radius('lg') expected 12.0, got " + r);
    exit;
}

// Test Border Width
let b = theme::border("thick");
if b != 4.0 {
    print("Error: theme::border('thick') expected 4.0, got " + b);
    exit;
}

// Test Z-Index
let z = theme::z("overlay");
if z != 50 {
    print("Error: theme::z('overlay') expected 50, got " + z);
    exit;
}

print("All theme tests passed!");
