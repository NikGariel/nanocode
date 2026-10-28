export type RGB = number | `#${string}`;
export type FormatId = 0 | 1 | 2 | 3 | 4;
export interface Palette {one?: RGB; zero?: RGB; background?: RGB}
export interface Format {id: number; name: 'S'|'M'|'L'|'XL'; capacity: number; rings: number}
export interface Options extends Palette {format?: FormatId; logoDataUrl?: string}
export interface Detection {bytes: Uint8Array; text: string; format: number; formatName: string; corrected: number; corners: number[]}
export interface NanoCode {
  formats: Format[];
  formatFor(data: string|Uint8Array, id?: FormatId): Format;
  assess(palette?: Palette): {flags: number; score: number; allowed: boolean; recommended: boolean; issues: string[]};
  backgrounds(one: RGB, zero: RGB): {color: string; score: number}[];
  svg(data: string|Uint8Array, options?: Options): string;
  scan(gray: Uint8Array, width: number, height: number): Detection|null;
}
export function loadNanoCode(source?: string|URL|Uint8Array|ArrayBuffer|WebAssembly.Module): Promise<NanoCode>;
