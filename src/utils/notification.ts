import { createApp, type App } from 'vue';
import Notification from '../components/Notification.vue';

// 通知类型接口
export interface NotificationOptions {
  title: string;
  message: string;
  duration?: number;
}

// 创建通知实例
let notificationInstance: any = null;
let app: App | null = null;

// 定义通知类型
type NotificationType = 'info' | 'success' | 'warning' | 'error' | 'debug';

// 初始化通知组件
const initNotification = () => {
  if (!notificationInstance) {
    // 创建容器元素
    const container = document.createElement('div');
    container.id = 'notification-container';
    document.body.appendChild(container);

    // 创建并挂载通知组件
    app = createApp(Notification);
    notificationInstance = app.mount(container);
  }
};

// 通用通知方法
const showNotification = (type: NotificationType, title: string, message: string, duration?: number) => {
  initNotification();
  notificationInstance.addNotification(type, title, message, duration);
};

// 显示信息通知
export const showInfo = (title: string, message: string, duration?: number) => {
  showNotification('info', title, message, duration);
};

// 显示成功通知
export const showSuccess = (title: string, message: string, duration?: number) => {
  showNotification('success', title, message, duration);
};

// 显示警告通知
export const showWarning = (title: string, message: string, duration?: number) => {
  showNotification('warning', title, message, duration);
};

// 显示错误通知
export const showError = (title: string, message: string, duration?: number) => {
  showNotification('error', title, message, duration);
};

// 显示调试通知
export const showDebug = (title: string, message: string, duration?: number) => {
  showNotification('debug', title, message, duration);
};

// 销毁通知实例（可选）
export const destroyNotification = () => {
  if (app && notificationInstance) {
    app.unmount();
    const container = document.getElementById('notification-container');
    if (container) {
      document.body.removeChild(container);
    }
    app = null;
    notificationInstance = null;
  }
};