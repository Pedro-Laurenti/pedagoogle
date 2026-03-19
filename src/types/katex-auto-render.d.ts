declare module "katex/contrib/auto-render" {
  interface Delimiter {
    left: string;
    right: string;
    display: boolean;
  }
  interface AutoRenderOptions {
    delimiters?: Delimiter[];
    throwOnError?: boolean;
    [key: string]: unknown;
  }
  function renderMathInElement(el: HTMLElement, options?: AutoRenderOptions): void;
  export default renderMathInElement;
}
