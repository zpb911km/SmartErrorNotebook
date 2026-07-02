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
    remark = db.Column(db.String(256), nullable=True, default='')
    created_at = db.Column(db.BigInteger, nullable=False, default=lambda: int(datetime.now().timestamp() * 1000))
    updated_at = db.Column(db.BigInteger, nullable=False, default=lambda: int(datetime.now().timestamp() * 1000))

    def to_dict(self):
        return {
            'id': self.id,
            'auth_key': self.auth_key,
            'remark': self.remark,
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


class SharedQuestion(db.Model):
    """分享错题表 - 用户分享到社区的错题"""
    __tablename__ = 'shared_questions'

    id = db.Column(db.String(64), primary_key=True, default=lambda: str(uuid.uuid4()))
    source_question_id = db.Column(db.String(64), unique=True, nullable=False, index=True)
    prompt = db.Column(db.Text, nullable=False)
    type_ = db.Column(db.String(64), nullable=False)
    answer = db.Column(db.Text, nullable=True, default='')
    analysis = db.Column(db.Text, nullable=True, default='')
    error_note = db.Column(db.Text, nullable=True, default='')
    sender_auth_key = db.Column(db.String(64), nullable=False, index=True)
    created_at = db.Column(db.BigInteger, nullable=False, default=lambda: int(datetime.now().timestamp() * 1000))

    def to_public_dict(self):
        """转换为公开格式（不含 sender_auth_key）"""
        return {
            'id': self.id,
            'prompt': self.prompt,
            'type_': self.type_,
            'answer': self.answer or '',
            'analysis': self.analysis or '',
            'error_note': self.error_note or '',
            'created_at': self.created_at,
        }


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
    table_name = data.get('table_name')

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
            # 客户端已决策，直接覆盖
            existing.version = data.get('version', 0)
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


# ==================== 社区分享 ====================

@app.route('/api/share/publish', methods=['POST'])
def share_publish():
    """
    分享错题到社区

    Request Body:
        auth_key: 用户认证 key
        id: 题目原始 UUID（用于排重和撤回匹配）
        prompt: 题目文本
        type_: 题型
        answer: 标准答案 (可选)
        analysis: 解析 (可选)
        error_note: 错题笔记 (可选)

    Returns:
        {success: true} 或 {error: ...}
    """
    data = request.get_json() or {}
    auth_key = data.get('auth_key')
    question_id = data.get('id')

    if not auth_key:
        return jsonify({'error': 'Missing auth_key'}), 400
    if not question_id:
        return jsonify({'error': 'Missing id'}), 400

    # 验证 auth_key
    user = UserAuth.query.filter_by(auth_key=auth_key).first()
    if not user:
        return jsonify({'error': 'Invalid auth_key'}), 401

    # 检查是否已分享，已存在则幂等返回
    existing = SharedQuestion.query.filter_by(source_question_id=question_id).first()
    if existing:
        return jsonify({'success': True})

    # 创建分享记录
    shared = SharedQuestion(
        source_question_id=question_id,
        prompt=data.get('prompt', ''),
        type_=data.get('type_', ''),
        answer=data.get('answer', ''),
        analysis=data.get('analysis', ''),
        error_note=data.get('error_note', ''),
        sender_auth_key=auth_key,
    )
    try:
        db.session.add(shared)
        db.session.commit()
        return jsonify({'success': True}), 201
    except Exception as e:
        db.session.rollback()
        return jsonify({'error': str(e)}), 500


@app.route('/api/share/list', methods=['GET'])
def share_list():
    """
    获取社区分享列表（分页，需鉴权）

    Query Params:
        auth_key: 用户认证 key
        page: 页码 (默认 1)
        page_size: 每页条数 (默认 20)

    Returns:
        {items: [...], has_more: bool}
    """
    auth_key = request.args.get('auth_key')
    if not auth_key:
        return jsonify({'error': 'Missing auth_key'}), 400

    # 验证 auth_key - 仅注册用户可浏览
    user = UserAuth.query.filter_by(auth_key=auth_key).first()
    if not user:
        return jsonify({'error': 'Invalid auth_key'}), 401

    page = request.args.get('page', 1, type=int)
    page_size = request.args.get('page_size', 20, type=int)
    page_size = min(page_size, 50)  # 上限保护

    query = SharedQuestion.query.order_by(SharedQuestion.created_at.desc())
    pagination = query.paginate(page=page, per_page=page_size, error_out=False)

    items = [item.to_public_dict() for item in pagination.items]
    return jsonify({
        'items': items,
        'has_more': pagination.has_next,
    })


@app.route('/api/share/revoke', methods=['POST'])
def share_revoke():
    """
    撤回分享的错题

    Request Body:
        auth_key: 用户认证 key
        id: 题目的原始 UUID（source_question_id）

    Returns:
        {success: true} 或 {error: ...}
    """
    data = request.get_json() or {}
    auth_key = data.get('auth_key')
    question_id = data.get('id')

    if not auth_key:
        return jsonify({'error': 'Missing auth_key'}), 400
    if not question_id:
        return jsonify({'error': 'Missing id'}), 400

    # 验证 auth_key
    user = UserAuth.query.filter_by(auth_key=auth_key).first()
    if not user:
        return jsonify({'error': 'Invalid auth_key'}), 401

    # 删除分享记录（需匹配 sender_auth_key，防止 A 撤回 B 的分享）
    try:
        deleted = SharedQuestion.query.filter_by(
            source_question_id=question_id,
            sender_auth_key=auth_key,
        ).delete()
        db.session.commit()
        return jsonify({'success': True, 'deleted': deleted > 0})
    except Exception as e:
        db.session.rollback()
        return jsonify({'error': str(e)}), 500


@app.route('/api/share/check', methods=['GET'])
def share_check():
    """
    检查指定题目是否已分享（用于 Detail 页初始化状态）

    Query Params:
        auth_key: 用户认证 key
        id: 题目的原始 UUID（source_question_id）

    Returns:
        {shared: bool}
    """
    auth_key = request.args.get('auth_key')
    question_id = request.args.get('id')

    if not auth_key:
        return jsonify({'error': 'Missing auth_key'}), 400
    if not question_id:
        return jsonify({'error': 'Missing id'}), 400

    # 验证 auth_key
    user = UserAuth.query.filter_by(auth_key=auth_key).first()
    if not user:
        return jsonify({'error': 'Invalid auth_key'}), 401

    existing = SharedQuestion.query.filter_by(
        source_question_id=question_id,
        sender_auth_key=auth_key,
    ).first()
    return jsonify({'shared': existing is not None})


# ==================== 管理后台 ====================

class AdminSession(db.Model):
    """管理员 session 表"""
    __tablename__ = 'admin_sessions'

    id = db.Column(db.String(64), primary_key=True, default=lambda: str(uuid.uuid4()))
    token = db.Column(db.String(64), unique=True, nullable=False, index=True)
    expires_at = db.Column(db.BigInteger, nullable=False)
    created_at = db.Column(db.BigInteger, nullable=False, default=lambda: int(datetime.now().timestamp() * 1000))


def calc_admin_code():
    """生成当前时间的 4 位动态验证码"""
    t = datetime.now(timezone(timedelta(hours=8))).timetuple()
    return '%02d%02d' % (t[3] + t[4], t[2])


def require_admin():
    """验证请求头中的 admin token"""
    token = request.headers.get('X-Admin-Token', '')
    if not token:
        return None
    session = AdminSession.query.filter_by(token=token).first()
    now = int(datetime.now().timestamp() * 1000)
    if not session or session.expires_at < now:
        return None
    return token


@app.route('/admin', methods=['GET'])
def admin_page():
    """管理后台页面"""
    return render_template('admin.html')


@app.route('/api/admin/verify', methods=['POST'])
def admin_verify():
    """验证管理动态码，成功后返回 session token"""
    data = request.get_json() or {}
    if data.get('code') != calc_admin_code():
        return jsonify({'valid': False}), 401

    token = str(uuid.uuid4())
    expiry = int(datetime.now().timestamp() * 1000) + 10 * 60 * 1000
    session = AdminSession(token=token, expires_at=expiry)
    try:
        db.session.add(session)
        db.session.commit()
        return jsonify({'valid': True, 'token': token})
    except Exception as e:
        db.session.rollback()
        return jsonify({'error': str(e)}), 500


@app.route('/api/admin/keys', methods=['GET', 'POST'])
def admin_keys():
    """管理授权码"""
    if not require_admin():
        return jsonify({'error': 'Unauthorized'}), 401

    if request.method == 'GET':
        users = UserAuth.query.order_by(UserAuth.created_at.desc()).all()
        return jsonify({'keys': [u.to_dict() for u in users]})

    # POST: 生成新授权码
    body = request.get_json() or {}
    auth_key = str(uuid.uuid4())
    remark = body.get('remark', '')
    user = UserAuth(auth_key=auth_key, remark=remark)
    try:
        db.session.add(user)
        db.session.commit()
        return jsonify({'auth_key': auth_key, 'remark': remark}), 201
    except Exception as e:
        db.session.rollback()
        return jsonify({'error': str(e)}), 500


@app.route('/api/admin/keys/<key_id>', methods=['PUT'])
def admin_update_key(key_id):
    """更新授权码备注"""
    if not require_admin():
        return jsonify({'error': 'Unauthorized'}), 401

    user = db.session.get(UserAuth, key_id)
    if not user:
        return jsonify({'error': 'Key not found'}), 404

    body = request.get_json() or {}
    if 'remark' in body:
        user.remark = body['remark']
        user.updated_at = int(datetime.now().timestamp() * 1000)

    try:
        db.session.commit()
        return jsonify({'success': True, 'auth_key': user.to_dict()})
    except Exception as e:
        db.session.rollback()
        return jsonify({'error': str(e)}), 500


@app.route('/api/admin/keys/<key_id>', methods=['DELETE'])
def admin_delete_key(key_id):
    """删除授权码"""
    if not require_admin():
        return jsonify({'error': 'Unauthorized'}), 401

    user = db.session.get(UserAuth, key_id)
    if not user:
        return jsonify({'error': 'Key not found'}), 404

    try:
        # 清理该用户的所有同步数据
        Record.query.filter_by(auth_key=user.auth_key).delete()
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
    app.run(debug=True, host='0.0.0.0', port=60032)
