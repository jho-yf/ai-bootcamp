import type { Slide } from './slide'
import type { StyleConfig } from './style'

export interface PresentationSummary {
  slug: string
  title: string
  slideCount: number
  totalCost: number
  createdAt: string
  updatedAt: string
}

export interface PresentationDetail {
  slug: string
  title: string
  style: StyleConfig
  slides: Slide[]
  totalCost: number
}

export interface GenerateProgress {
  status: 'generating' | 'complete' | 'error'
  progress?: number
  image?: Slide
  error?: string
}

export interface BatchGenerateProgress {
  sid: string
  status: 'generating' | 'complete' | 'error'
  progress?: number
  image?: Slide
  error?: string
}

export interface CreatePresentationRequest {
  slug: string
  title: string
}

export interface UpdateTitleRequest {
  title: string
}

export interface CreateSlideRequest {
  content: string
  index?: number
}

export interface UpdateSlideRequest {
  content?: string
  activeImageIndex?: number
}

export interface UpdateSlideResponse {
  slide: Slide
  hasMatchingImage: boolean
}

export interface ReorderSlidesRequest {
  orderedSids: string[]
}

export interface GenerateStyleRequest {
  prompt: string
}

export interface GenerateStyleResponse {
  candidates: string[]
  prompt: string
}

export interface SelectStyleRequest {
  referenceImage: string
}

export interface SelectStyleResponse {
  style: StyleConfig
}

export interface BatchGenerateRequest {
  sids: string[]
}
