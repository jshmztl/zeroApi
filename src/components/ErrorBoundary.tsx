import * as React from "react";
import { AlertTriangle, RotateCw, Home } from "lucide-react";

interface Props {
  children: React.ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error("[ErrorBoundary]", error, info);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="h-screen w-screen flex items-center justify-center bg-gray-50 dark:bg-gray-950">
          <div className="text-center max-w-md px-6">
            <div className="w-16 h-16 mx-auto mb-4 rounded-2xl bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
              <AlertTriangle className="h-8 w-8 text-red-500" />
            </div>
            <h1 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
              应用发生异常
            </h1>
            <p className="text-sm text-gray-500 dark:text-gray-400 mb-1">
              渲染过程中出现了未预期的错误，请尝试以下操作：
            </p>
            <div className="text-xs text-gray-400 dark:text-gray-500 mb-6 font-mono bg-gray-100 dark:bg-gray-800 rounded-lg p-3 text-left overflow-auto max-h-32">
              {this.state.error?.message || "未知错误"}
            </div>
            <div className="flex items-center justify-center gap-3">
              <button
                onClick={this.handleReset}
                className="inline-flex items-center gap-1.5 px-4 py-2 text-sm font-medium bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors"
              >
                <RotateCw className="h-4 w-4" />
                重试
              </button>
              <button
                onClick={() => {
                  this.handleReset();
                  window.location.hash = "#/";
                  window.location.reload();
                }}
                className="inline-flex items-center gap-1.5 px-4 py-2 text-sm font-medium border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
              >
                <Home className="h-4 w-4" />
                回到首页
              </button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
