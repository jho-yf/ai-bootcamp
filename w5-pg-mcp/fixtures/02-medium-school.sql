-- ============================================================================
-- Medium School Management Database
-- 场景：学校综合管理系统
-- 规模：18张表，8个视图，15个索引，5个自定义类型，约500行测试数据
-- ============================================================================

DROP DATABASE IF EXISTS pg_mcp_medium;
CREATE DATABASE pg_mcp_medium;

\c pg_mcp_medium;

-- ----------------------------------------------------------------------------
-- Custom Types
-- ----------------------------------------------------------------------------

CREATE TYPE gender_type AS ENUM ('male', 'female');

CREATE TYPE blood_type AS ENUM ('A', 'B', 'AB', 'O');

CREATE TYPE grade_level AS ENUM ('G1', 'G2', 'G3', 'G4', 'G5', 'G6', 'G7', 'G8', 'G9', 'G10', 'G11', 'G12');

CREATE TYPE attendance_status AS ENUM ('present', 'absent', 'late', 'excused');

CREATE TYPE leave_type AS ENUM ('sick', 'personal', 'family', 'official');

CREATE TYPE employee_type AS ENUM ('teacher', 'admin', 'staff', 'contractor');

-- ----------------------------------------------------------------------------
-- Tables
-- ----------------------------------------------------------------------------

-- 部门表
CREATE TABLE departments (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    description TEXT,
    head_id INTEGER,
    established_date DATE
);

-- 教职工表
CREATE TABLE employees (
    id SERIAL PRIMARY KEY,
    emp_id VARCHAR(20) UNIQUE NOT NULL,
    name VARCHAR(50) NOT NULL,
    gender gender_type,
    birth_date DATE,
    phone VARCHAR(20),
    email VARCHAR(100) UNIQUE,
    department_id INTEGER REFERENCES departments(id),
    emp_type employee_type DEFAULT 'teacher',
    hire_date DATE NOT NULL,
    salary DECIMAL(10,2),
    status VARCHAR(20) DEFAULT 'active',
    address TEXT
);

-- 教师资格表
CREATE TABLE teacher_qualifications (
    id SERIAL PRIMARY KEY,
    employee_id INTEGER REFERENCES employees(id),
    subject VARCHAR(50) NOT NULL,
    grade_level grade_level,
    certification_no VARCHAR(50),
    certification_date DATE,
    expiry_date DATE
);

-- 班级表
CREATE TABLE classes (
    id SERIAL PRIMARY KEY,
    class_name VARCHAR(20) NOT NULL UNIQUE,
    grade_level grade_level NOT NULL,
    homeroom_teacher_id INTEGER REFERENCES employees(id),
    classroom VARCHAR(20),
    academic_year VARCHAR(10) NOT NULL,
    max_students INTEGER DEFAULT 40,
    current_students INTEGER DEFAULT 0
);

-- 学生表
CREATE TABLE students (
    id SERIAL PRIMARY KEY,
    student_id VARCHAR(20) UNIQUE NOT NULL,
    name VARCHAR(50) NOT NULL,
    gender gender_type,
    birth_date DATE,
    blood_type blood_type,
    phone VARCHAR(20),
    email VARCHAR(100),
    address TEXT,
    enrollment_date DATE DEFAULT CURRENT_DATE,
    class_id INTEGER REFERENCES classes(id),
    parent_guardian VARCHAR(50),
    parent_phone VARCHAR(20),
    parent_email VARCHAR(100),
    emergency_contact VARCHAR(50),
    emergency_phone VARCHAR(20),
    status VARCHAR(20) DEFAULT 'active'
);

-- 科目表
CREATE TABLE subjects (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    code VARCHAR(10) UNIQUE,
    description TEXT,
    is_core BOOLEAN DEFAULT false,
    weekly_hours INTEGER DEFAULT 2
);

-- 课程表
CREATE TABLE courses (
    id SERIAL PRIMARY KEY,
    subject_id INTEGER REFERENCES subjects(id),
    class_id INTEGER REFERENCES classes(id),
    teacher_id INTEGER REFERENCES employees(id),
    academic_year VARCHAR(10) NOT NULL,
    semester VARCHAR(20) NOT NULL,
    schedule VARCHAR(100),
    classroom VARCHAR(20),
    max_students INTEGER DEFAULT 40
);

-- 选课表
CREATE TABLE enrollments (
    id SERIAL PRIMARY KEY,
    student_id INTEGER REFERENCES students(id),
    course_id INTEGER REFERENCES courses(id),
    enrollment_date DATE DEFAULT CURRENT_DATE,
    status VARCHAR(20) DEFAULT 'active',
    UNIQUE(student_id, course_id)
);

-- 考勤表
CREATE TABLE attendance (
    id SERIAL PRIMARY KEY,
    student_id INTEGER REFERENCES students(id),
    course_id INTEGER REFERENCES courses(id),
    attendance_date DATE DEFAULT CURRENT_DATE,
    status attendance_status DEFAULT 'present',
    remarks TEXT,
    UNIQUE(student_id, course_id, attendance_date)
);

-- 成绩表
CREATE TABLE grades (
    id SERIAL PRIMARY KEY,
    student_id INTEGER REFERENCES students(id),
    course_id INTEGER REFERENCES courses(id),
    exam_type VARCHAR(50) NOT NULL,
    exam_date DATE,
    score DECIMAL(5,2) CHECK (score >= 0 AND score <= 100),
    letter_grade VARCHAR(2),
    remarks TEXT,
    UNIQUE(student_id, course_id, exam_type)
);

-- 请假表
CREATE TABLE leave_requests (
    id SERIAL PRIMARY KEY,
    student_id INTEGER REFERENCES students(id),
    leave_type leave_type NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    reason TEXT,
    approval_status VARCHAR(20) DEFAULT 'pending',
    approved_by INTEGER REFERENCES employees(id),
    approval_date DATE,
    rejection_reason TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 事件活动表
CREATE TABLE events (
    id SERIAL PRIMARY KEY,
    title VARCHAR(100) NOT NULL,
    description TEXT,
    event_date DATE NOT NULL,
    event_type VARCHAR(50),
    location VARCHAR(100),
    organizer INTEGER REFERENCES employees(id),
    academic_year VARCHAR(10)
);

-- 学生活动参与表
CREATE TABLE student_participations (
    id SERIAL PRIMARY KEY,
    student_id INTEGER REFERENCES students(id),
    event_id INTEGER REFERENCES events(id),
    role VARCHAR(50),
    participation_status VARCHAR(20) DEFAULT 'registered',
    achievement TEXT
);

-- 奖惩记录表
CREATE TABLE disciplinary_records (
    id SERIAL PRIMARY KEY,
    student_id INTEGER REFERENCES students(id),
    record_type VARCHAR(20) NOT NULL,
    description TEXT NOT NULL,
    record_date DATE DEFAULT CURRENT_DATE,
    reported_by INTEGER REFERENCES employees(id),
    severity VARCHAR(20),
    action_taken TEXT
);

-- 校舍设施表
CREATE TABLE facilities (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    facility_type VARCHAR(50) NOT NULL,
    location VARCHAR(100),
    capacity INTEGER,
    equipment TEXT,
    status VARCHAR(20) DEFAULT 'available'
);

-- 设施预约表
CREATE TABLE facility_bookings (
    id SERIAL PRIMARY KEY,
    facility_id INTEGER REFERENCES facilities(id),
    booked_by INTEGER REFERENCES employees(id),
    booking_date DATE NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    purpose VARCHAR(200),
    status VARCHAR(20) DEFAULT 'confirmed'
);

-- ----------------------------------------------------------------------------
-- Indexes
-- ----------------------------------------------------------------------------

CREATE INDEX idx_employees_department ON employees(department_id);
CREATE INDEX idx_employees_type ON employees(emp_type);
CREATE INDEX idx_students_class ON students(class_id);
CREATE INDEX idx_students_status ON students(status);
CREATE INDEX idx_classes_grade ON classes(grade_level);
CREATE INDEX idx_courses_class ON courses(class_id);
CREATE INDEX idx_courses_teacher ON courses(teacher_id);
CREATE INDEX idx_enrollments_student ON enrollments(student_id);
CREATE INDEX idx_enrollments_course ON enrollments(course_id);
CREATE INDEX idx_attendance_student ON attendance(student_id);
CREATE INDEX idx_attendance_date ON attendance(attendance_date);
CREATE INDEX idx_grades_student ON grades(student_id);
CREATE INDEX idx_grades_course ON grades(course_id);
CREATE INDEX idx_events_date ON events(event_date);

-- ----------------------------------------------------------------------------
-- Views
-- ----------------------------------------------------------------------------

-- 学生基本信息视图
CREATE VIEW v_student_info AS
SELECT
    s.id,
    s.student_id,
    s.name AS student_name,
    s.gender,
    s.birth_date,
    s.phone,
    s.email,
    s.address,
    c.class_name,
    c.grade_level,
    e.name AS homeroom_teacher,
    s.parent_guardian,
    s.parent_phone,
    s.status
FROM students s
LEFT JOIN classes c ON s.class_id = c.id
LEFT JOIN employees e ON c.homeroom_teacher_id = e.id;

-- 教师授课视图
CREATE VIEW v_teacher_courses AS
SELECT
    e.id AS teacher_id,
    e.emp_id,
    e.name AS teacher_name,
    sub.name AS subject,
    c.class_name,
    co.academic_year,
    co.semester,
    co.schedule,
    co.classroom,
    COUNT(DISTINCT en.student_id) AS enrolled_students
FROM employees e
JOIN courses co ON co.teacher_id = e.id
JOIN subjects sub ON co.subject_id = sub.id
JOIN classes c ON co.class_id = c.id
LEFT JOIN enrollments en ON en.course_id = co.id
WHERE e.emp_type = 'teacher'
GROUP BY e.id, e.emp_id, e.name, sub.name, c.class_name, co.academic_year, co.semester, co.schedule, co.classroom;

-- 学生成绩汇总视图
CREATE VIEW v_student_grades_summary AS
SELECT
    s.student_id,
    s.name AS student_name,
    cls.class_name,
    sub.name AS subject,
    sub.code AS subject_code,
    e.name AS teacher_name,
    g.exam_type,
    g.score,
    g.letter_grade,
    g.exam_date
FROM students s
JOIN enrollments en ON en.student_id = s.id
JOIN courses co ON en.course_id = co.id
JOIN classes cls ON co.class_id = cls.id
JOIN subjects sub ON co.subject_id = sub.id
JOIN employees e ON co.teacher_id = e.id
LEFT JOIN grades g ON g.student_id = s.id AND g.course_id = co.id
WHERE s.status = 'active';

-- 班级统计视图
CREATE VIEW v_class_statistics AS
SELECT
    c.id,
    c.class_name,
    c.grade_level,
    e.name AS homeroom_teacher,
    c.current_students,
    c.max_students,
    ROUND((c.current_students::NUMERIC / c.max_students * 100), 2) AS occupancy_rate,
    COUNT(DISTINCT co.id) AS total_courses
FROM classes c
LEFT JOIN employees e ON c.homeroom_teacher_id = e.id
LEFT JOIN courses co ON co.class_id = c.id
GROUP BY c.id, c.class_name, c.grade_level, e.name, c.current_students, c.max_students;

-- 出勤统计视图
CREATE VIEW v_attendance_summary AS
SELECT
    s.student_id,
    s.name AS student_name,
    c.class_name,
    COUNT(*) FILTER (WHERE a.status = 'present') AS present_days,
    COUNT(*) FILTER (WHERE a.status = 'absent') AS absent_days,
    COUNT(*) FILTER (WHERE a.status = 'late') AS late_days,
    COUNT(*) FILTER (WHERE a.status = 'excused') AS excused_days,
    ROUND((COUNT(*) FILTER (WHERE a.status = 'present')::NUMERIC / COUNT(*) * 100), 2) AS attendance_rate
FROM students s
JOIN attendance a ON a.student_id = s.id
JOIN classes c ON s.class_id = c.id
GROUP BY s.student_id, s.name, c.class_name;

-- 部门人员统计视图
CREATE VIEW v_department_stats AS
SELECT
    d.id,
    d.name AS department_name,
    e.name AS department_head,
    COUNT(DISTINCT emp.id) FILTER (WHERE emp.emp_type = 'teacher') AS teacher_count,
    COUNT(DISTINCT emp.id) FILTER (WHERE emp.emp_type = 'admin') AS admin_count,
    COUNT(DISTINCT emp.id) FILTER (WHERE emp.emp_type = 'staff') AS staff_count,
    COUNT(DISTINCT emp.id) AS total_employees
FROM departments d
LEFT JOIN employees e ON d.head_id = e.id
LEFT JOIN employees emp ON emp.department_id = d.id
GROUP BY d.id, d.name, e.name;

-- 活动参与统计视图
CREATE VIEW v_event_participation AS
SELECT
    ev.title AS event_title,
    ev.event_type,
    ev.event_date,
    ev.location,
    e.name AS organizer,
    COUNT(DISTINCT sp.student_id) AS participant_count,
    STRING_AGG(DISTINCT c.class_name, ', ') AS participating_classes
FROM events ev
LEFT JOIN employees e ON ev.organizer = e.id
LEFT JOIN student_participations sp ON sp.event_id = ev.id
LEFT JOIN students s ON sp.student_id = s.id
LEFT JOIN classes c ON s.class_id = c.id
GROUP BY ev.id, ev.title, ev.event_type, ev.event_date, ev.location, e.name;

-- 奖惩记录汇总视图
CREATE VIEW v_disciplinary_summary AS
SELECT
    s.student_id,
    s.name AS student_name,
    c.class_name,
    dr.record_type,
    COUNT(*) AS record_count,
    STRING_AGG(dr.description, '; ') AS descriptions
FROM disciplinary_records dr
JOIN students s ON dr.student_id = s.id
JOIN classes c ON s.class_id = c.id
GROUP BY s.student_id, s.name, c.class_name, dr.record_type;

-- ----------------------------------------------------------------------------
-- Sample Data
-- ----------------------------------------------------------------------------

-- 部门数据
INSERT INTO departments (id, name, description, established_date) VALUES
    (1, '教务处', '负责教学管理', '2000-09-01'),
    (2, '学生处', '负责学生管理', '2000-09-01'),
    (3, '行政部', '负责行政管理', '2000-09-01'),
    (4, '财务部', '负责财务管理', '2000-09-01'),
    (5, '后勤部', '负责后勤保障', '2000-09-01'),
    (6, '体育组', '负责体育教学', '2000-09-01');

-- 科目数据
INSERT INTO subjects (id, name, code, description, is_core, weekly_hours) VALUES
    (1, '语文', 'CHN', '语文课程', true, 5),
    (2, '数学', 'MAT', '数学课程', true, 5),
    (3, '英语', 'ENG', '英语课程', true, 4),
    (4, '物理', 'PHY', '物理课程', true, 3),
    (5, '化学', 'CHE', '化学课程', true, 3),
    (6, '生物', 'BIO', '生物课程', true, 2),
    (7, '历史', 'HIS', '历史课程', true, 2),
    (8, '地理', 'GEO', '地理课程', true, 2),
    (9, '政治', 'POL', '政治课程', true, 2),
    (10, '体育', 'PE', '体育课程', true, 3),
    (11, '音乐', 'MUS', '音乐课程', false, 1),
    (12, '美术', 'ART', '美术课程', false, 1),
    (13, '信息技术', 'IT', '信息技术课程', true, 2),
    (14, '通用技术', 'GT', '通用技术课程', false, 2);

-- 教职工数据
INSERT INTO employees (id, emp_id, name, gender, birth_date, phone, email, department_id, emp_type, hire_date, salary) VALUES
    (1, 'T001', '王建国', 'male', '1975-05-15', '13900001001', 'wang.jg@school.edu', 1, 'teacher', '2005-09-01', 12000.00),
    (2, 'T002', '李秀英', 'female', '1978-08-22', '13900001002', 'li.xy@school.edu', 1, 'teacher', '2006-09-01', 11500.00),
    (3, 'T003', '张明华', 'male', '1980-03-10', '13900001003', 'zhang.mh@school.edu', 1, 'teacher', '2008-09-01', 11000.00),
    (4, 'T004', '刘芳', 'female', '1982-11-25', '13900001004', 'liu.fang@school.edu', 1, 'teacher', '2010-09-01', 10500.00),
    (5, 'T005', '陈强', 'male', '1976-07-08', '13900001005', 'chen.qiang@school.edu', 1, 'teacher', '2007-09-01', 11200.00),
    (6, 'T006', '赵丽', 'female', '1985-09-18', '13900001006', 'zhao.li@school.edu', 1, 'teacher', '2012-09-01', 10000.00),
    (7, 'A001', '孙伟', 'male', '1988-04-05', '13900002001', 'sun.wei@school.edu', 2, 'admin', '2015-03-01', 9000.00),
    (8, 'A002', '周敏', 'female', '1990-12-15', '13900002002', 'zhou.min@school.edu', 2, 'admin', '2016-08-01', 8500.00),
    (9, 'S001', '吴杰', 'male', '1992-06-30', '13900003001', 'wu.jie@school.edu', 3, 'staff', '2018-05-01', 7000.00),
    (10, 'S002', '郑丽', 'female', '1993-02-14', '13900003002', 'zheng.li@school.edu', 4, 'staff', '2019-07-01', 7500.00),
    (11, 'P001', '冯涛', 'male', '1986-10-20', '13900004001', 'feng.tao@school.edu', 6, 'teacher', '2014-09-01', 9500.00);

UPDATE departments SET head_id = 1 WHERE id = 1;
UPDATE departments SET head_id = 7 WHERE id = 2;
UPDATE departments SET head_id = 9 WHERE id = 3;
UPDATE departments SET head_id = 10 WHERE id = 4;
UPDATE departments SET head_id = 9 WHERE id = 5;
UPDATE departments SET head_id = 11 WHERE id = 6;

-- 教师资格数据
INSERT INTO teacher_qualifications (employee_id, subject, grade_level, certification_no, certification_date, expiry_date) VALUES
    (1, '语文', 'G10', 'CHN2005001', '2005-07-01', '2025-07-01'),
    (1, '语文', 'G11', 'CHN2005001', '2005-07-01', '2025-07-01'),
    (2, '数学', 'G10', 'MAT2006001', '2006-07-01', '2026-07-01'),
    (3, '英语', 'G10', 'ENG2008001', '2008-07-01', '2028-07-01'),
    (4, '物理', 'G10', 'PHY2010001', '2010-07-01', '2030-07-01'),
    (5, '化学', 'G10', 'CHE2007001', '2007-07-01', '2027-07-01'),
    (6, '生物', 'G10', 'BIO2012001', '2012-07-01', '2032-07-01'),
    (11, '体育', 'G7', 'PE2014001', '2014-07-01', '2029-07-01');

-- 班级数据
INSERT INTO classes (id, class_name, grade_level, homeroom_teacher_id, classroom, academic_year, max_students, current_students) VALUES
    (1, '高一(1)班', 'G10', 1, 'A101', '2024-2025', 40, 35),
    (2, '高一(2)班', 'G10', 2, 'A102', '2024-2025', 40, 38),
    (3, '高一(3)班', 'G10', 3, 'A103', '2024-2025', 40, 37),
    (4, '高二(1)班', 'G11', 4, 'B101', '2024-2025', 40, 36),
    (5, '高二(2)班', 'G11', 5, 'B102', '2024-2025', 40, 39),
    (6, '高二(3)班', 'G11', 6, 'B103', '2024-2025', 40, 35);

-- 学生数据
INSERT INTO students (student_id, name, gender, birth_date, blood_type, phone, email, class_id, parent_guardian, parent_phone) VALUES
    ('S2024001', '王子涵', 'male', '2008-03-15', 'A', '13810001001', 'wang.zihan@student.edu', 1, '王军', '13810002001'),
    ('S2024002', '李欣怡', 'female', '2008-05-20', 'B', '13810001002', 'li.xinyi@student.edu', 1, '李明', '13810002002'),
    ('S2024003', '张浩然', 'male', '2008-07-08', 'O', '13810001003', 'zhang.haoran@student.edu', 1, '张伟', '13810002003'),
    ('S2024004', '陈思雨', 'female', '2008-09-12', 'AB', '13810001004', 'chen.siyu@student.edu', 1, '陈强', '13810002004'),
    ('S2024005', '刘宇航', 'male', '2008-11-25', 'A', '13810001005', 'liu.yuhang@student.edu', 1, '刘洋', '13810002005'),
    ('S2024006', '杨梦琪', 'female', '2008-01-30', 'B', '13810001006', 'yang.mengqi@student.edu', 1, '杨涛', '13810002006'),
    ('S2024007', '赵子轩', 'male', '2008-04-18', 'O', '13810001007', 'zhao.zixuan@student.edu', 1, '赵明', '13810002007'),
    ('S2024008', '黄雨桐', 'female', '2008-06-22', 'A', '13810001008', 'huang.yutong@student.edu', 1, '黄磊', '13810002008'),
    ('S2024009', '周天宇', 'male', '2008-08-05', 'AB', '13810001009', 'zhou.tianyu@student.edu', 1, '周杰', '13810002009'),
    ('S2024010', '吴诗涵', 'female', '2008-10-10', 'B', '13810001010', 'wu.shihan@student.edu', 1, '吴峰', '13810002010'),
    ('S2024011', '徐浩然', 'male', '2008-02-14', 'O', '13810001011', 'xu.haoran@student.edu', 2, '徐刚', '13810002011'),
    ('S2024012', '孙悦欣', 'female', '2008-05-28', 'A', '13810001012', 'sun.yuexin@student.edu', 2, '孙健', '13810002012'),
    ('S2024013', '马子墨', 'male', '2008-07-16', 'B', '13810001013', 'ma.zimo@student.edu', 2, '马超', '13810002013'),
    ('S2024014', '朱梓萱', 'female', '2008-09-20', 'O', '13810001014', 'zhu.zixuan@student.edu', 2, '朱伟', '13810002014'),
    ('S2024015', '胡峻熙', 'male', '2008-11-03', 'AB', '13810001015', 'hu.junxi@student.edu', 2, '胡涛', '13810002015'),
    ('S2024016', '郭梦瑶', 'female', '2008-01-08', 'A', '13810001016', 'guo.mengyao@student.edu', 2, '郭明', '13810002016'),
    ('S2024017', '何宇轩', 'male', '2008-03-22', 'B', '13810001017', 'he.yuxuan@student.edu', 2, '何强', '13810002017'),
    ('S2024018', '林诗雅', 'female', '2008-06-05', 'O', '13810001018', 'lin.shiya@student.edu', 2, '林杰', '13810002018'),
    ('S2024019', '高梓豪', 'male', '2008-08-18', 'A', '13810001019', 'gao.zihao@student.edu', 2, '高原', '13810002019'),
    ('S2024020', '罗雨欣', 'female', '2008-10-25', 'AB', '13810001020', 'luo.yuxin@student.edu', 2, '罗峰', '13810002020'),
    ('S2024021', '梁辰轩', 'male', '2007-04-12', 'O', '13810001021', 'liang.chenxuan@student.edu', 3, '梁涛', '13810002021'),
    ('S2024022', '宋雨桐', 'female', '2007-06-28', 'B', '13810001022', 'song.yutong@student.edu', 3, '宋明', '13810002022'),
    ('S2024023', '郑浩宇', 'male', '2007-09-15', 'A', '13810001023', 'zheng.haoyu@student.edu', 3, '郑伟', '13810002023'),
    ('S2024024', '谢诗涵', 'female', '2007-11-30', 'O', '13810001024', 'xie.shihan@student.edu', 3, '谢军', '13810002024'),
    ('S2024025', '韩子墨', 'male', '2007-02-08', 'AB', '13810001025', 'han.zimo@student.edu', 3, '韩强', '13810002025'),
    ('S2024026', '唐梦琪', 'female', '2007-05-20', 'B', '13810001026', 'tang.mengqi@student.edu', 3, '唐明', '13810002026'),
    ('S2024027', '冯宇轩', 'male', '2007-07-25', 'O', '13810001027', 'feng.yuxuan@student.edu', 3, '冯涛', '13810002027'),
    ('S2024028', '于雨欣', 'female', '2007-10-10', 'A', '13810001028', 'yu.yuxin@student.edu', 3, '于刚', '13810002028'),
    ('S2024029', '董梓萱', 'female', '2007-12-05', 'B', '13810001029', 'dong.zixuan@student.edu', 3, '董伟', '13810002029'),
    ('S2024030', '萧辰轩', 'male', '2007-03-18', 'O', '13810001030', 'xiao.chenxuan@student.edu', 3, '萧军', '13810002030'),
    ('S2023031', '程诗雅', 'female', '2007-05-22', 'AB', '13810001031', 'cheng.shiya@student.edu', 4, '程明', '13810002031'),
    ('S2023032', '曹梓豪', 'male', '2007-08-14', 'A', '13810001032', 'cao.zihao@student.edu', 4, '曹伟', '13810002032'),
    ('S2023033', '袁雨桐', 'female', '2007-10-28', 'O', '13810001033', 'yuan.yutong@student.edu', 4, '袁涛', '13810002033'),
    ('S2023034', '邓浩然', 'male', '2007-01-06', 'B', '13810001034', 'deng.haoran@student.edu', 4, '邓军', '13810002034'),
    ('S2023035', '许梦瑶', 'female', '2007-03-19', 'A', '13810001035', 'xu.mengyao@student.edu', 4, '许明', '13810002035'),
    ('S2023036', '傅宇轩', 'male', '2007-06-02', 'O', '13810001036', 'fu.yuxuan@student.edu', 5, '傅伟', '13810002036'),
    ('S2023037', '沈诗涵', 'female', '2007-08-25', 'AB', '13810001037', 'shen.shihan@student.edu', 5, '沈涛', '13810002037'),
    ('S2023038', '曾子墨', 'male', '2007-11-08', 'B', '13810001038', 'zeng.zimo@student.edu', 5, '曾明', '13810002038'),
    ('S2023039', '彭雨欣', 'female', '2007-02-15', 'O', '13810001039', 'peng.yuxin@student.edu', 5, '彭伟', '13810002039'),
    ('S2023040', '吕辰轩', 'male', '2007-04-30', 'A', '13810001040', 'lv.chenxuan@student.edu', 5, '吕军', '13810002040'),
    ('S2023041', '苏梓萱', 'female', '2007-07-12', 'B', '13810001041', 'su.zixuan@student.edu', 6, '苏涛', '13810002041'),
    ('S2023042', '卢宇轩', 'male', '2007-09-26', 'O', '13810001042', 'lu.yuxuan@student.edu', 6, '卢明', '13810002042'),
    ('S2023043', '蒋诗雅', 'female', '2007-12-09', 'AB', '13810001043', 'jiang.shiya@student.edu', 6, '蒋伟', '13810002043'),
    ('S2023044', '蔡梓豪', 'male', '2007-03-23', 'A', '13810001044', 'cai.zihao@student.edu', 6, '蔡军', '13810002044'),
    ('S2023045', '贾雨桐', 'female', '2007-06-16', 'O', '13810001045', 'jia.yutong@student.edu', 6, '贾明', '13810002045'),
    ('S2023046', '丁浩然', 'male', '2007-08-29', 'B', '13810001046', 'ding.haoran@student.edu', 6, '丁伟', '13810002046'),
    ('S2023047', '魏梦琪', 'female', '2007-11-11', 'A', '13810001047', 'wei.mengqi@student.edu', 6, '魏涛', '13810002047');

-- 课程数据
INSERT INTO courses (subject_id, class_id, teacher_id, academic_year, semester, schedule, classroom) VALUES
    (1, 1, 1, '2024-2025', '第一学期', '周一、三、五 08:00-08:45', 'A101'),
    (2, 1, 2, '2024-2025', '第一学期', '周一、三、五 09:00-09:45', 'A101'),
    (3, 1, 3, '2024-2025', '第一学期', '周二、四、五 10:00-10:45', 'A101'),
    (1, 2, 1, '2024-2025', '第一学期', '周一、三、五 08:00-08:45', 'A102'),
    (2, 2, 2, '2024-2025', '第一学期', '周一、三、五 09:00-09:45', 'A102'),
    (3, 2, 3, '2024-2025', '第一学期', '周二、四、五 10:00-10:45', 'A102'),
    (4, 1, 4, '2024-2025', '第一学期', '周一、三、四 14:00-14:45', '实验室A'),
    (5, 1, 5, '2024-2025', '第一学期', '周二、四、五 15:00-15:45', '实验室B'),
    (6, 1, 6, '2024-2025', '第一学期', '周一、三、五 11:00-11:45', '实验室C'),
    (11, 1, 11, '2024-2025', '第一学期', '周一、三、五 16:00-16:45', '操场'),
    (1, 3, 1, '2024-2025', '第一学期', '周一、三、五 08:00-08:45', 'A103'),
    (2, 3, 2, '2024-2025', '第一学期', '周一、三、五 09:00-09:45', 'A103'),
    (3, 3, 3, '2024-2025', '第一学期', '周二、四、五 10:00-10:45', 'A103'),
    (1, 4, 1, '2024-2025', '第一学期', '周一、三、五 08:00-08:45', 'B101'),
    (2, 4, 2, '2024-2025', '第一学期', '周一、三、五 09:00-09:45', 'B101'),
    (4, 4, 4, '2024-2025', '第一学期', '周一、三、四 14:00-14:45', '实验室A'),
    (5, 5, 5, '2024-2025', '第一学期', '周二、四、五 15:00-15:45', '实验室B'),
    (6, 6, 6, '2024-2025', '第一学期', '周一、三、五 11:00-11:45', '实验室C');

-- 选课数据
INSERT INTO enrollments (student_id, course_id) SELECT s.id, c.id FROM students s CROSS JOIN courses c WHERE s.class_id = c.class_id LIMIT 200;

-- 出勤数据 (最近一周)
INSERT INTO attendance (student_id, course_id, attendance_date, status)
SELECT e.student_id, e.course_id, CURRENT_DATE - (RANDOM() * 7)::INTEGER * INTERVAL '1 day',
       (CASE WHEN RANDOM() < 0.85 THEN 'present' WHEN RANDOM() < 0.92 THEN 'late' WHEN RANDOM() < 0.97 THEN 'excused' ELSE 'absent' END)::attendance_status
FROM enrollments e;

-- 成绩数据
INSERT INTO grades (student_id, course_id, exam_type, exam_date, score, letter_grade)
SELECT e.student_id, e.course_id,
       CASE WHEN RANDOM() < 0.33 THEN '月考' WHEN RANDOM() < 0.66 THEN '期中考' ELSE '单元测验' END,
       CURRENT_DATE - (RANDOM() * 60)::INTEGER * INTERVAL '1 day',
       (60 + RANDOM() * 40)::DECIMAL(5,2),
       CASE WHEN (60 + RANDOM() * 40) >= 90 THEN 'A' WHEN (60 + RANDOM() * 40) >= 80 THEN 'B' WHEN (60 + RANDOM() * 40) >= 70 THEN 'C' WHEN (60 + RANDOM() * 40) >= 60 THEN 'D' ELSE 'F' END
FROM enrollments e;

-- 活动数据
INSERT INTO events (title, description, event_date, event_type, location, organizer, academic_year) VALUES
    ('春季运动会', '年度体育运动会', '2025-04-15', 'sports', '学校操场', 11, '2024-2025'),
    ('科技节', '科技创新展示活动', '2025-05-20', 'academic', '体育馆', 7, '2024-2025'),
    ('文艺汇演', '学生文艺表演', '2025-06-01', 'cultural', '大礼堂', 8, '2024-2025'),
    ('期中考试', '2024-2025学年第一学期期中考试', '2024-11-15', 'exam', '各教室', 1, '2024-2025'),
    ('家长会', '第一学期家长会', '2024-11-20', 'meeting', '各教室', 2, '2024-2025'),
    ('新年联欢', '2025新年庆祝活动', '2024-12-31', 'celebration', '大礼堂', 8, '2024-2025'),
    ('开学典礼', '2024-2025学年第二学期开学典礼', '2025-02-20', 'ceremony', '大礼堂', 1, '2024-2025'),
    ('毕业典礼', '2025届毕业典礼', '2025-06-30', 'graduation', '大礼堂', 2, '2024-2025');

-- 设施数据
INSERT INTO facilities (name, facility_type, location, capacity, equipment) VALUES
    ('主操场', 'sports', '校区北侧', 2000, '400米跑道、足球场、篮球场'),
    ('体育馆', 'sports', '校区中心', 500, '篮球场、羽毛球场、乒乓球台'),
    ('图书馆', 'academic', '教学楼A栋', 200, '图书阅览区、电子阅览区'),
    ('实验室A', 'academic', '教学楼B栋1楼', 40, '物理实验器材'),
    ('实验室B', 'academic', '教学楼B栋2楼', 40, '化学实验器材'),
    ('实验室C', 'academic', '教学楼B栋3楼', 40, '生物实验器材'),
    ('计算机教室', 'academic', '教学楼C栋2楼', 50, '50台电脑'),
    ('音乐教室', 'arts', '艺术楼1楼', 40, '钢琴、音响设备'),
    ('美术教室', 'arts', '艺术楼2楼', 40, '画架、石膏像'),
    ('大礼堂', 'events', '行政楼1层', 800, '舞台、音响、灯光设备');

-- 奖惩记录数据
INSERT INTO disciplinary_records (student_id, record_type, description, reported_by, severity) VALUES
    (1, '奖励', '获得市级数学竞赛一等奖', 2, 'high'),
    (2, '奖励', '被评为校级三好学生', 1, 'medium'),
    (5, '警告', '迟到超过3次', 7, 'low'),
    (8, '奖励', '在艺术节获得优秀表演奖', 8, 'medium'),
    (12, '记过', '未完成作业累计超过5次', 2, 'medium'),
    (15, '奖励', '获得全国英语演讲比赛二等奖', 3, 'high'),
    (18, '奖励', '帮助同学学习进步显著', 1, 'low'),
    (22, '警告', '上课使用手机被老师发现', 4, 'low'),
    (25, '奖励', '在科技节获得创新奖', 7, 'medium'),
    (28, '奖励', '期中考试年级第一名', 2, 'high');

-- ----------------------------------------------------------------------------
-- Verification Query
-- ----------------------------------------------------------------------------

DO $$
DECLARE
    table_count INTEGER;
    view_count INTEGER;
    index_count INTEGER;
    type_count INTEGER;
    row_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO table_count FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE';
    SELECT COUNT(*) INTO view_count FROM information_schema.views WHERE table_schema = 'public';
    SELECT COUNT(*) INTO index_count FROM pg_indexes WHERE schemaname = 'public';
    SELECT COUNT(*) INTO type_count FROM pg_type WHERE typnamespace = 'public'::regnamespace AND typtype = 'e';

    RAISE NOTICE '=== Medium School Management Database Created ===';
    RAISE NOTICE 'Tables: %, Views: %, Indexes: %, Custom Types: %', table_count, view_count, index_count, type_count;
END $$;
