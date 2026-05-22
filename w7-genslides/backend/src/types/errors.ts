export const ErrorCodes = {
  SLIDE_NOT_FOUND: 'SLIDE_NOT_FOUND',
  PRESENTATION_NOT_FOUND: 'PRESENTATION_NOT_FOUND',
  GENERATION_FAILED: 'GENERATION_FAILED',
  INVALID_REQUEST: 'INVALID_REQUEST',
} as const;

export type ErrorCode = (typeof ErrorCodes)[keyof typeof ErrorCodes];

export class AppError extends Error {
  constructor(
    public code: ErrorCode,
    public statusCode: number,
    message: string,
  ) {
    super(message);
    this.name = 'AppError';
  }
}
