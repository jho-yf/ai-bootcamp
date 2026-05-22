export interface SlideImage {
  filename: string;
  textHash: string;
  createdAt: string;
  cost: number;
}

export interface Slide {
  sid: string;
  content: string;
  images: SlideImage[];
  activeImageIndex: number;
}

export interface StyleConfig {
  prompt: string;
  candidates: string[];
  referenceImage: string | null;
}

export interface Presentation {
  slug: string;
  title: string;
  style: StyleConfig;
  slides: Slide[];
  totalCost: number;
}

export interface PresentationSummary {
  slug: string;
  title: string;
  slideCount: number;
  totalCost: number;
  createdAt: string;
  updatedAt: string;
}

export interface GenerateProgress {
  status: 'generating' | 'complete' | 'error';
  progress?: number;
  image?: SlideImage;
  error?: string;
}

export interface BatchGenerateProgress extends GenerateProgress {
  sid: string;
}
