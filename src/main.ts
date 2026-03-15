const target = new URLSearchParams(window.location.search).get("window");

if (target === "indicator") {
  await import("./windows/indicator/main");
} else {
  await import("./windows/settings/main");
}

export {};
