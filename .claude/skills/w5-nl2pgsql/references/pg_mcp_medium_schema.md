# pg_mcp_medium - 学校管理系统数据库 Schema

## 概述

学校综合管理系统，包含18张表、8个视图、15+个索引、6个自定义类型。

## 自定义类型 (Enums)

### gender_type
性别：`male`, `female`

### grade_level
年级：`G1`, `G2`, `G3`, `G4`, `G5`, `G6`, `G7`, `G8`, `G9`, `G10`, `G11`, `G12`

### attendance_status
考勤状态：`present`, `absent`, `late`, `excused`

### leave_type
请假类型：`sick`, `personal`, `family`, `official`

### blood_type
血型：`A`, `B`, `AB`, `O`

### employee_type
员工类型：`teacher`, `admin`, `staff`, `contractor`

## 表结构 (Tables)

### students
学生表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `student_id`: character varying (NOT NULL) - 学号
- `name`: character varying (NOT NULL) - 姓名
- `gender`: gender_type - 性别
- `birth_date`: date - 出生日期
- `blood_type`: blood_type - 血型
- `phone`: character varying - 电话
- `email`: character varying - 邮箱
- `address`: text - 地址
- `class_id`: integer - 班级ID (外键: classes.id)
- `enrollment_date`: date - 入学日期
- `parent_guardian`: character varying - 家长/监护人
- `parent_phone`: character varying - 家长电话
- `parent_email`: character varying - 家长邮箱
- `emergency_contact`: character varying - 紧急联系人
- `emergency_phone`: character varying - 紧急联系电话
- `status`: character varying - 状态

### employees
员工表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `emp_id`: character varying (NOT NULL) - 员工ID
- `name`: character varying (NOT NULL) - 姓名
- `birth_date`: date - 出生日期
- `gender`: gender_type - 性别
- `phone`: character varying - 电话
- `email`: character varying - 邮箱
- `address`: text - 地址
- `hire_date`: date (NOT NULL) - 入职日期
- `salary`: numeric - 薪资
- `department_id`: integer - 部门ID (外键: departments.id)
- `emp_type`: employee_type - 员工类型
- `status`: character varying - 状态

### classes
班级表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `grade_level`: grade_level (NOT NULL) - 年级
- `class_name`: character varying (NOT NULL) - 班级名称
- `academic_year`: character varying (NOT NULL) - 学年
- `classroom`: character varying - 教室
- `homeroom_teacher_id`: integer - 班主任ID (外键: employees.id)
- `max_students`: integer - 最大学生数
- `current_students`: integer - 当前学生数

### subjects
科目表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `name`: character varying (NOT NULL) - 科目名称
- `code`: character varying - 科目代码
- `description`: text - 描述
- `weekly_hours`: integer - 每周课时
- `is_core`: boolean - 是否核心科目

### courses
课程表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `class_id`: integer - 班级ID (外键: classes.id)
- `subject_id`: integer - 科目ID (外键: subjects.id)
- `teacher_id`: integer - 教师ID (外键: employees.id)
- `academic_year`: character varying (NOT NULL) - 学年
- `semester`: character varying (NOT NULL) - 学期
- `classroom`: character varying - 教室
- `schedule`: character varying - 排课安排
- `max_students`: integer - 最大学生数

### departments
部门表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `name`: character varying (NOT NULL) - 部门名称
- `description`: text - 描述
- `head_id`: integer - 部门负责人ID (外键: employees.id)
- `established_date`: date - 建立日期

### grades
成绩表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `student_id`: integer - 学生ID (外键: students.id)
- `course_id`: integer - 课程ID (外键: courses.id)
- `exam_type`: character varying (NOT NULL) - 考试类型
- `exam_date`: date - 考试日期
- `score`: numeric - 分数
- `letter_grade`: character varying - 等级评分
- `remarks`: text - 备注

### attendance
考勤表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `student_id`: integer - 学生ID (外键: students.id)
- `course_id`: integer - 课程ID (外键: courses.id)
- `attendance_date`: date - 考勤日期
- `status`: attendance_status - 考勤状态
- `remarks`: text - 备注

### leave_requests
请假申请表
- `id`: integer (NOT NULL, PRIMARY KEY)
- ``student_id`: integer - 学生ID (外键: students.id)
- `leave_type`: leave_type (NOT NULL) - 请假类型
- `start_date`: date (NOT NULL) - 开始日期
- `end_date`: date (NOT NULL) - 结束日期
- `reason`: text - 请假原因
- `approval_status`: character varying - 审批状态
- `approved_by`: integer - 审批人ID (外键: employees.id)
- `approval_date`: date - 审批日期
- `rejection_reason`: text - 拒绝原因
- `created_at`: timestamp without time zone - 创建时间

### enrollments
选课表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `student_id`: integer - 学生ID (外键: students.id)
- `course_id`: integer - 课程ID (外键: courses.id)
- `enrollment_date`: date - 选课日期
- `status`: character varying - 状态

### disciplinary_records
违纪记录表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `student_id`: integer - 学生ID (外键: students.id)
- `record_type`: character varying (NOT NULL) - 记录类型
- `description`: text (NOT NULL) - 描述
- `record_date`: date - 记录日期
- `reported_by`: integer - 报告人ID (外键: employees.id)
- `severity`: character varying - 严重程度
- `action_taken`: text - 采取的行动

### events
活动表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `title`: character varying (NOT NULL) - 活动标题
- `description`: text - 描述
- `event_date`: date (NOT NULL) - 活动日期
- `academic_year`: character varying - 学年
- `organizer`: integer - 组织者ID (外键: employees.id)
- `event_type`: character varying - 活动类型
- `location`: character varying - 地点

### facilities
设施表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `name`: character varying (NOT NULL) - 设施名称
- `facility_type`: character varying (NOT NULL) - 设施类型
- `location`: character varying - 位置
- `capacity`: integer - 容量
- `equipment`: text - 设备
- `status`: character varying - 状态

### facility_bookings
设施预订表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `facility_id`: integer - 设施ID (外键: facilities.id)
- `booked_by`: integer - 预订人ID (外键: employees.id)
- `booking_date`: date (NOT NULL) - 预订日期
- `start_time`: time without time zone (NOT NULL) - 开始时间
- `end_time`: time without time zone (NOT NULL) - 结束时间
- `purpose`: character varying - 用途
- `status`: character varying - 状态

### student_participations
学生参与活动表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `student_id`: integer - 学生ID (外键: students.id)
- `event_id`: integer - 活动ID (外键: events.id)
- `role`: character varying - 角色
- `participation_status`: character varying - 参与状态
- `achievement`: text - 成就

### teacher_qualifications
教师资质表
- `id`: integer (NOT NULL, PRIMARY KEY)
- `employee_id`: integer - 员工ID (外键: employees.id)
- `subject`: character varying (NOT NULL) - 科目
- `grade_level`: grade_level - 年级
- `certification_no`: character varying - 认证编号
- `certification_date`: date - 认证日期
- `expiry_date`: date - 过期日期

## 视图 (Views)

### v_student_info
学生基本信息视图

### v_teacher_courses
教师授课视图

### v_student_grades_summary
成绩汇总视图

### v_class_statistics
班级统计视图

### v_attendance_summary
出勤统计视图

### v_department_stats
部门统计视图

### v_disciplinary_summary
违纪统计视图

### v_event_participation
活动参与视图

## 核心业务关系

1. 学生通过 enrollments 选修课程
2. 教师通过 courses 授课
3. 成绩记录在 grades 表
4. 考勤记录在 attendance 表
5. 请假通过 leave_requests 申请
6. 学生可参与多个活动 (student_participations)
7. 设施通过 facility_bookings 预订
