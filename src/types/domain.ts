export enum InputFileType {
  Image = "Image",
  Exe = "Exe",
  Lnk = "Lnk",
  Directory = "Directory",
}

export enum IconShape {
  Rectangle = "Rectangle",
  RoundedRectangle = "RoundedRectangle",
  Squircle = "Squircle",
}

export enum HqPanelShape {
  Rect = "Rect",
  Squircle = "Squircle",
  Circle = "Circle",
}

export type HqShadowMode = "icon" | "badge";

export enum BackplateType {
  None = "None",
  Solid = "Solid",
  Gradient = "Gradient",
}

export enum ForegroundFit {
  Contain = "Contain",
  Cover = "Cover",
}

export enum ExportMode {
  ExportAsIco = "ExportAsIco",
  ExportAsPng = "ExportAsPng",
  ApplyToLnk = "ApplyToLnk",
}

export interface OuterShadowConfig {
  enabled: boolean;
  offsetX: number;
  offsetY: number;
  blurRadius: number;
  spread: number;
  color: string;
}

export interface StrokeConfig {
  width: number;
  color: string;
}

/**
 * Edge gloss: a rim light that hugs the shape boundary and fades inwards,
 * modulated by two soft bands along a rotatable light axis.
 */
export interface GlossConfig {
  enabled: boolean;
  /** Rim falloff distance in px. */
  width: number;
  strength: number;
  lightColor: string;
  /** Intensity of the band on the lit side of the light axis. */
  topStrength: number;
  /** Intensity of the band on the opposite side. */
  bottomStrength: number;
  /** How far the lit band reaches from the light side, in px. */
  topDecay: number;
  /** How far the opposite band reaches, in px. */
  bottomDecay: number;
  /** Rotation of the light axis in degrees (0 = light from the top). */
  angleDegrees: number;
}

export interface AutoCutoutConfig {
  enabled: boolean;
  tolerance: number;
  feather: number;
}

export interface HqRenderConfig {
  enabled: boolean;
  thresh: number;
  iconRatio: number;
  bg: number;
  lightMix: number;
  darkMix: number;
  gloss: number;
  iconLight: number;
  corner: number;
  shape: HqPanelShape;
  offsetX: number;
  offsetY: number;
  customBgEnabled: boolean;
  customBgColor: string;
  shadowOpacity: number;
  shadowBlurFactor: number;
  shadowOffsetFactor: number;
  shadowFade: number;
  shadowMode: HqShadowMode;
}

export interface RenderConfig {
  foregroundScalePercent: number;
  foregroundFit: ForegroundFit;
  foregroundOffsetX: number;
  foregroundOffsetY: number;
  foregroundRotationDegrees: number;
  foregroundOpacityPercent: number;
  canvasInset: number;
  contentScalePercent: number;
  shape: IconShape;
  cornerRadius: number;
  squircleExponent: number;
  backplateType: BackplateType;
  backplateColor: string;
  gradientStartColor: string;
  gradientEndColor: string;
  gradientAngleDegrees: number;
  outerShadow: OuterShadowConfig;
  stroke: StrokeConfig;
  gloss: GlossConfig;
  /** Saturation of the imported source image, in percent. 0 = grayscale. */
  sourceSaturationPercent: number;
  autoCutout: AutoCutoutConfig;
  hqRender: HqRenderConfig;
}

export interface InputItem {
  id: string;
  sourcePath: string;
  displayName: string;
  fileType: InputFileType;
  parentDirectoryPath: string | null;
  resolvedTargetPath: string | null;
  sourceWidth: number;
  sourceHeight: number;
  thumbnailPngBase64: string;
  supportedExportModes: ExportMode[];
  warnings: string[];
}

export interface UpscaleConfig {
  enabled: boolean;
  scale: number;
  denoise: number;
  model: string;
}

export interface BrushPoint {
  x: number;
  y: number;
}

export interface WandStroke {
  points: BrushPoint[];
  tolerance: number;
}

export interface EraserStroke {
  points: BrushPoint[];
  size: number;
  hardness: number;
}

export interface WandSelection {
  x: number;
  y: number;
  tolerance: number;
  maskPngBase64: string;
}

export interface BrushStroke {
  points: BrushPoint[];
  color: string;
  size: number;
  opacity: number;
  mode: BrushMode;
  clipToMask: boolean;
}

export type BrushMode = "paint" | "erase";

export type ColorPickTarget =
  | "brush"
  | "backplate"
  | "gradientStart"
  | "gradientEnd"
  | "shadow"
  | "stroke"
  | "glossLight";

export type PreviewTool = "none" | "brush" | "eraser" | "eyedropper" | "magic-wand" | "source-eraser";

export interface ItemConfig {
  renderConfig: RenderConfig;
  upscaleConfig: UpscaleConfig;
  brushStrokes: BrushStroke[];
  brushColor: string;
  brushSize: number;
  brushMode: BrushMode;
  brushClipToMask: boolean;
  wandStrokes: WandStroke[];
  wandTolerance: number;
  eraserStrokes: EraserStroke[];
  eraserSize: number;
  eraserHardness: number;
  activePresetId: string | null;
}

export interface PresetDefinition {
  id: string;
  name: string;
  description: string;
  config: RenderConfig;
  /** User-created presets are renamable and deletable. */
  custom?: boolean;
}
