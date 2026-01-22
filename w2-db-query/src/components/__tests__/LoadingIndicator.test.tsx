import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { LoadingIndicator } from "../LoadingIndicator";

describe("LoadingIndicator", () => {
  it("应该在不加载时不渲染", () => {
    const { container } = render(<LoadingIndicator loading={false} />);
    expect(container.firstChild).toBeNull();
  });

  it("应该在加载时显示加载指示器", () => {
    render(<LoadingIndicator loading={true} />);
    expect(screen.getByText(/查询执行中/i)).toBeInTheDocument();
  });

  it("应该显示自定义消息", () => {
    render(<LoadingIndicator loading={true} message="自定义加载消息" />);
    expect(screen.getByText("自定义加载消息")).toBeInTheDocument();
  });

  it("应该在提供 onCancel 时显示取消按钮", () => {
    const onCancel = () => {};
    render(<LoadingIndicator loading={true} onCancel={onCancel} />);
    expect(screen.getByText(/取消查询/i)).toBeInTheDocument();
  });

  it("应该在不提供 onCancel 时不显示取消按钮", () => {
    render(<LoadingIndicator loading={true} />);
    expect(screen.queryByText(/取消查询/i)).not.toBeInTheDocument();
  });
});
