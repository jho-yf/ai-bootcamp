import { NextRequest, NextResponse } from 'next/server';
import { handleError } from '../../../src/middleware/error-handler';
import { presentationService } from '../../../src/lib/services';
import type { CreatePresentationBody } from '../../../src/types/api';

export async function GET() {
  try {
    const presentations = await presentationService.list();
    return NextResponse.json({ presentations });
  } catch (error) {
    return handleError(error);
  }
}

export async function POST(request: NextRequest) {
  try {
    const body: CreatePresentationBody = await request.json();
    const result = await presentationService.create(body.title);
    return NextResponse.json(result, { status: 201 });
  } catch (error) {
    return handleError(error);
  }
}
