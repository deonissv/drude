export const addSliderListener = (sliderId: string, inputId: string) => {
  const slider = document.querySelector(sliderId) as HTMLInputElement;
  const input = document.querySelector(inputId) as HTMLInputElement;

  slider.addEventListener('input', (e) => {
    input.value = (e.target as HTMLInputElement).value;
  });

  input.addEventListener('input', (e) => {
    slider.value = (e.target as HTMLInputElement).value;
  });

}