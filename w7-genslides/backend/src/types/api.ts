import type { Presentation, PresentationSummary, Slide, SlideImage, StyleConfig } from '../../../shared/src/types';

export interface ApiResponse<T = unknown> {
  data?: T;
  error?: { code: string; message: string };
}

export interface CreatePresentationBody {
  title: string;
}

export interface UpdatePresentationBody {
  title: string;
}

export interface CreateSlideBody {
  content: string;
  index?: number;
}

export interface UpdateSlideBody {
  content?: string;
  activeImageIndex?: number;
}

export interface ReorderSlidesBody {
  orderedSids: string[];
}

export interface BatchGenerateBody {
  sids: string[];
}

export interface GenerateStyleBody {
  prompt: string;
}

export interface SelectStyleBody {
  referenceImage: string | null;
}

export interface SlidesResponse {
  title: string;
  style: StyleConfig;
  slides: Slide[];
  totalCost: number;
}

export interface CreateSlideResponse {
  slide: Slide;
}

export interface UpdateSlideResponse {
  slide: Slide;
  hasMatchingImage: boolean;
}

export interface ReorderSlidesResponse {
  slides: Slide[];
}

export interface GenerateStyleResponse {
  candidates: string[];
  prompt: string;
}

export interface SelectStyleResponse {
  style: StyleConfig;
}
