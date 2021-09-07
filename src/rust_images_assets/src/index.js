import { rust_images } from "../../declarations/rust_images";

document.getElementById("clickMeBtn").addEventListener("click", async () => {
  const name = document.getElementById("name").value.toString();
  // Interact with rust_images actor, calling the greet method
  const greeting = await rust_images.greet(name);

  document.getElementById("greeting").innerText = greeting;
});
