"""
多设备同步服务器 - 单体 Flask 应用

支持错题记录的多设备同步，通过 auth_key 进行用户识别和分片。
"""

from flask import Flask, request, jsonify, render_template
from flask_sqlalchemy import SQLAlchemy
from datetime import datetime, timezone, timedelta
import uuid
import os

# ==================== 配置 ====================
app = Flask(__name__)

# 默认使用 SQLite，可通过环境变量配置为 PostgreSQL/MySQL
DB_TYPE = os.getenv('DB_TYPE', 'sqlite')
DB_PATH = os.getenv('DB_PATH', './sync_data.db')
SECRET_KEY = os.getenv('SECRET_KEY', 'dev-secret-key-change-in-production')

if DB_TYPE == 'postgresql':
    app.config['SQLALCHEMY_DATABASE_URI'] = os.getenv('DATABASE_URL', 'postgresql://localhost/sync')
elif DB_TYPE == 'mysql':
    app.config['SQLALCHEMY_DATABASE_URI'] = os.getenv('DATABASE_URL', 'mysql://localhost/sync')
else:
    # SQLite
    app.config['SQLALCHEMY_DATABASE_URI'] = f'sqlite:///{DB_PATH}'

app.config['SQLALCHEMY_TRACK_MODIFICATIONS'] = False
app.config['SECRET_KEY'] = SECRET_KEY

db = SQLAlchemy(app)


# ==================== 数据模型 ====================

class UserAuth(db.Model):
    """用户认证表 - 存储有效的 auth_key"""
    __tablename__ = 'user_auth'

    id = db.Column(db.String(64), primary_key=True, default=lambda: str(uuid.uuid4()))
    auth_key = db.Column(db.String(64), unique=True, nullable=False, index=True)
    created_at = db.Column(db.BigInteger, nullable=False, default=lambda: int(datetime.now().timestamp() * 1000))
    updated_at = db.Column(db.BigInteger, nullable=False, default=lambda: int(datetime.now().timestamp() * 1000))

    def to_dict(self):
        return {
            'id': self.id,
            'auth_key': self.auth_key,
            'created_at': self.created_at,
        }


class Record(db.Model):
    """同步记录表 - 按用户分片存储所有设备的数据副本"""
    __tablename__ = 'records'

    id = db.Column(db.String(64), primary_key=True)  # 记录 ID（UUID）
    table_name = db.Column(db.String(64), nullable=False, index=True)  # 表名，用于识别记录类型
    auth_key = db.Column(db.String(64), nullable=False, index=True)  # 用户标识

    # 同步字段
    version = db.Column(db.Integer, nullable=False, default=0, index=True)
    status = db.Column(db.String(16), nullable=False, default='synced')  # synced / pending
    deleted_at = db.Column(db.BigInteger, nullable=True)  # 软删除时间戳，None 表示未删除

    # 数据字段 - 存储所有业务字段的 JSON
    data = db.Column(db.JSON, nullable=False)

    # 时间戳
    created_at = db.Column(db.BigInteger, nullable=False, default=lambda: int(datetime.now().timestamp() * 1000))
    updated_at = db.Column(db.BigInteger, nullable=False, default=lambda: int(datetime.now().timestamp() * 1000), onupdate=lambda: int(datetime.now().timestamp() * 1000))

    def to_dict(self):
        """转换为客户端格式（不含 auth_key）"""
        return {
            'id': self.id,
            'table_name': self.table_name,
            'version': self.version,
            'status': self.status,
            'deleted_at': self.deleted_at,
            'updated_at': self.updated_at,
            'data': self.data,
        }

    def to_header(self):
        """转换为轻量格式（不含 data），供握手使用"""
        return {
            'id': self.id,
            'table_name': self.table_name,
            'version': self.version,
            'status': self.status,
            'deleted_at': self.deleted_at,
            'updated_at': self.updated_at,
        }

    @classmethod
    def from_client_dict(cls, record_dict, auth_key, table_name):
        """从客户端字典创建记录（用于更新时保留原 ID）"""
        return cls(
            id=record_dict['id'],
            table_name=table_name,
            auth_key=auth_key,
            version=record_dict.get('version', 0),
            status=record_dict.get('status', 'synced'),
            deleted_at=record_dict.get('deleted_at'),
            data=record_dict['data'],
            updated_at=int(datetime.now().timestamp() * 1000),
        )

    @classmethod
    def from_new_dict(cls, record_dict, auth_key, table_name):
        """从客户端字典创建新记录（生成新 ID）"""
        new_id = str(uuid.uuid4())
        now = int(datetime.now().timestamp() * 1000)
        return cls(
            id=new_id,
            table_name=table_name,
            auth_key=auth_key,
            version=record_dict.get('version', 0),
            status=record_dict.get('status', 'synced'),
            deleted_at=record_dict.get('deleted_at'),
            data=record_dict['data'],
            created_at=now,
            updated_at=now,
        )


# ==================== CORS ====================

@app.after_request
def cors_headers(response):
    response.headers['Access-Control-Allow-Origin'] = '*'
    response.headers['Access-Control-Allow-Headers'] = 'Content-Type,Authorization'
    response.headers['Access-Control-Allow-Methods'] = 'GET,POST,PUT,DELETE,OPTIONS'
    return response

@app.before_request
def handle_preflight():
    if request.method == 'OPTIONS':
        return jsonify({}), 200

# ==================== 路由 ====================

@app.route('/api/auth/validate', methods=['POST'])
def validate_auth():
    """验证 auth_key 是否有效"""
    data = request.get_json() or {}
    auth_key = data.get('auth_key')

    if not auth_key:
        return jsonify({'valid': False, 'error': 'Missing auth_key'}), 400

    user = UserAuth.query.filter_by(auth_key=auth_key).first()
    if user:
        return jsonify({'valid': True})
    return jsonify({'valid': False}), 401


@app.route('/api/auth/generate', methods=['POST'])
def generate_auth_key():
    """生成新的 auth_key（管理员功能）"""
    auth_key = str(uuid.uuid4())
    user = UserAuth(auth_key=auth_key)
    try:
        db.session.add(user)
        db.session.commit()
        return jsonify({'auth_key': auth_key}), 201
    except Exception as e:
        db.session.rollback()
        return jsonify({'error': str(e)}), 500


@app.route('/api/sync/get_all_sync_data', methods=['GET'])
def get_all_sync_data():
    """
    获取当前用户的所有同步数据（供客户端握手使用）

    Query Params:
        auth_key: 用户认证 key

    Returns:
        all_records: 服务端该用户的所有记录（作为 remote 数据）
    """
    auth_key = request.args.get('auth_key')
    if not auth_key:
        return jsonify({'error': 'Missing auth_key'}), 400

    # 验证 auth_key
    user = UserAuth.query.filter_by(auth_key=auth_key).first()
    if not user:
        return jsonify({'error': 'Invalid auth_key'}), 401

    # 获取用户所有记录（包含 table_name 用于前端识别）
    records = Record.query.filter_by(auth_key=auth_key).all()

    return jsonify({
        'all_records': [r.to_header() for r in records]
    })


@app.route('/api/sync/upload/<record_id>', methods=['POST'])
def upload_single_record(record_id):
    """
    上传单个记录（客户端→服务端）

    Path Params:
        record_id: 记录 ID

    Request Body:
        auth_key: 用户认证 key
        table_name: 表名 (error_questions, subjects, srs_data, attachments, error_tags, sources)
        version: 版本号
        status: 状态 (synced/pending)
        deleted_at: 软删除时间戳 (可选)
        data: 数据字段 JSON

    Returns:
        result: {success, new_version?, error?, conflict?}
    """
    data = request.get_json()
    auth_key = data.get('auth_key')
    table_name = data.get('table_name')  # 新增：表名
    client_version = data.get('version', 0)
    client_status = data.get('status', 'synced')

    if not auth_key:
        return jsonify({'error': 'Missing auth_key'}), 400

    if not table_name:
        return jsonify({'error': 'Missing table_name'}), 400

    # 验证 auth_key
    user = UserAuth.query.filter_by(auth_key=auth_key).first()
    if not user:
        return jsonify({'error': 'Invalid auth_key'}), 401

    now = int(datetime.now().timestamp() * 1000)

    try:
        # 检查记录是否存在
        existing = Record.query.filter_by(id=record_id, auth_key=auth_key, table_name=table_name).first()

        if existing:
            # 记录已存在 - 执行冲突检测
            # 协议：只有当 local.status == 'pending' 且 (remote 不存在 OR local.version == remote.version) 时才允许推送
            if client_status == 'pending' and client_version != existing.version:
                # CONFLICT: 客户端是 pending 状态但版本不匹配
                return jsonify({
                    'success': False,
                    'conflict': True,
                    'error': f'Conflict: client version {client_version} does not match server version {existing.version}',
                    'server_version': existing.version,
                }), 409

            # 无冲突：更新版本并覆盖数据
            existing.version = existing.version + 1
            existing.status = 'synced'
            existing.deleted_at = data.get('deleted_at')
            existing.data = data['data']
            existing.updated_at = now
            db.session.commit()
            return jsonify({
                'success': True,
                'new_version': existing.version,
            })
        else:
            # 记录不存在：创建新记录（新建或从其他设备拉取后的上传）
            new_record = Record(
                id=record_id,
                table_name=table_name,
                auth_key=auth_key,
                version=data.get('version', 0) + 1,
                status=data.get('status', 'synced'),
                deleted_at=data.get('deleted_at'),
                data=data['data'],
                created_at=now,
                updated_at=now,
            )
            db.session.add(new_record)
            db.session.commit()
            return jsonify({
                'success': True,
                'new_version': new_record.version,
            })

    except Exception as e:
        db.session.rollback()
        return jsonify({
            'success': False,
            'error': str(e),
        }), 500


@app.route('/api/sync/download/<record_id>', methods=['GET'])
def download_record(record_id):
    """
    根据记录 ID 下载单条记录

    Query Params:
        auth_key: 用户认证 key

    Returns:
        record: 记录详情（如果存在）
    """
    auth_key = request.args.get('auth_key')
    if not auth_key:
        return jsonify({'error': 'Missing auth_key'}), 400

    # 查询记录
    record = Record.query.filter_by(id=record_id, auth_key=auth_key).first()

    if not record:
        return jsonify({'error': 'Record not found'}), 404

    return jsonify({'record': record.to_dict()})


@app.route('/health', methods=['GET'])
def health_check():
    """健康检查端点"""
    return jsonify({'status': 'ok'})


# ==================== 管理后台 ====================

def calc_admin_code():
    """生成当前时间的 4 位动态验证码"""
    tz = timezone(timedelta(hours=8))
    now = datetime.now(tz)
    h, m, d = now.hour, now.minute, now.day
    return f"{h + m}{d:02d}"


@app.route('/admin', methods=['GET'])
def admin_page():
    """管理后台页面"""
    return render_template('admin.html')


@app.route('/api/admin/verify', methods=['POST'])
def admin_verify():
    """验证管理动态码"""
    data = request.get_json() or {}
    return jsonify({'valid': data.get('code') == calc_admin_code()})


@app.route('/api/admin/keys', methods=['GET', 'POST'])
def admin_keys():
    """管理授权码（需 X-Admin-Code 头验证）"""
    if request.headers.get('X-Admin-Code') != calc_admin_code():
        return jsonify({'error': 'Invalid admin code'}), 401

    if request.method == 'GET':
        users = UserAuth.query.order_by(UserAuth.created_at.desc()).all()
        return jsonify({'keys': [u.to_dict() for u in users]})

    # POST: 生成新授权码
    auth_key = str(uuid.uuid4())
    user = UserAuth(auth_key=auth_key)
    try:
        db.session.add(user)
        db.session.commit()
        return jsonify({'auth_key': auth_key}), 201
    except Exception as e:
        db.session.rollback()
        return jsonify({'error': str(e)}), 500


@app.route('/api/admin/keys/<key_id>', methods=['DELETE'])
def admin_delete_key(key_id):
    """删除授权码"""
    if request.headers.get('X-Admin-Code') != calc_admin_code():
        return jsonify({'error': 'Invalid admin code'}), 401

    user = db.session.get(UserAuth, key_id)
    if not user:
        return jsonify({'error': 'Key not found'}), 404

    try:
        db.session.delete(user)
        db.session.commit()
        return jsonify({'success': True})
    except Exception as e:
        db.session.rollback()
        return jsonify({'error': str(e)}), 500
def init_db():
    """初始化数据库"""
    with app.app_context():
        db.create_all()
        print("Database initialized successfully!")


if __name__ == '__main__':
    init_db()
    app.run(debug=True, host='0.0.0.0', port=5000)
