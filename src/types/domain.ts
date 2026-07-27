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

export enum BackplateType {
  None = "None",
  Solid = "Solid",
  Gradient = "Gradient",
}

export enum ExportMode {
  ExportAsIco = "ExportAsIco",
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

export interface RenderConfig {
  foregroundScalePercent: number;
  foregroundOffsetX: number;
  foregroundOffsetY: number;
  foregroundRotationDegrees: number;
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

export interface PresetDefinition {
  id: string;
  name: string;
  description: string;
  config: RenderConfig;
}
