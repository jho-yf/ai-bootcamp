import { NextResponse } from 'next/server';
import { AppError } from '../types/errors';

export function handleError(error: unknown): NextResponse {
  if (error instanceof AppError) {
    console.error(`[Error] code=${error.code} ${error.message}`);
    return NextResponse.json(
      { error: { code: error.code, message: error.message } },
      { status: error.statusCode },
    );
  }

  console.error('[Error] Unexpected error:', error);
  const message = error instanceof Error ? error.message : 'Internal server error';
  return NextResponse.json(
    { error: { code: 'INTERNAL_ERROR', message } },
    { status: 500 },
  );
}
