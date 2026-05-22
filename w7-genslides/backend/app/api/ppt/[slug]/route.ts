import { NextRequest, NextResponse } from 'next/server';
import { handleError } from '../../../../src/middleware/error-handler';
import { presentationService } from '../../../../src/lib/services';
import type { UpdatePresentationBody } from '../../../../src/types/api';

export async function GET(
  _request: NextRequest,
  { params }: { params: Promise<{ slug: string }> },
) {
  try {
    const { slug } = await params;
    const presentation = await presentationService.get(slug);
    return NextResponse.json(presentation);
  } catch (error) {
    return handleError(error);
  }
}

export async function PUT(
  request: NextRequest,
  { params }: { params: Promise<{ slug: string }> },
) {
  try {
    const { slug } = await params;
    const body: UpdatePresentationBody = await request.json();
    const result = await presentationService.update(slug, body.title);
    return NextResponse.json(result);
  } catch (error) {
    return handleError(error);
  }
}

export async function DELETE(
  _request: NextRequest,
  { params }: { params: Promise<{ slug: string }> },
) {
  try {
    const { slug } = await params;
    await presentationService.delete(slug);
    return new NextResponse(null, { status: 204 });
  } catch (error) {
    return handleError(error);
  }
}
