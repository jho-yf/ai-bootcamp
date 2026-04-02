# pg_mcp_large - 医院信息系统数据库 Schema

## 概述

综合医院信息系统 (HIS)，包含39+张表、14+个视图、40+个索引、22个自定义类型。

## 自定义类型 (Enums)

### gender_enum
性别：`male`, `female`, `other`

### blood_type_enum
血型：`A`, `B`, `AB`, `O`

### patient_type
患者类型：`outpatient`, `inpatient`, `emergency`

### registration_status
挂号状态：`pending`, `confirmed`, `completed`, `cancelled`

### appointment_status
预约状态：`scheduled`, `confirmed`, `completed`, `cancelled`, `no_show`

### bed_status
床位状态：`available`, `occupied`, `maintenance`, `reserved`

### severity_level
严重程度：`mild`, `moderate`, `severe`, `critical`

### priority_level
优先级：`low`, `normal`, `high`, `urgent`, `emergency`

### payment_method
付费方式：`cash`, `insurance`, `credit_card`, `debit_card`, `weixin`, `alipay`

### exam_status
检查状态：`ordered`, `scheduled`, `in_progress`, `completed`, `cancelled`

### prescription_status
处方状态：`pending`, `dispensed`, `cancelled`, `completed`

### surgery_status
手术状态：`scheduled`, `in_progress`, `completed`, `cancelled`, `postponed`

### anesthesia_type
麻醉类型：`local`, `general`, `spinal`, `epidural`, `none`

### staff_type
员工类型：`doctor`, `nurse`, `pharmacist`, `technician`, `admin`, `service`, `management`

### doctor_title
医生头衔：`resident`, `attending`, `chief`, `director`, `professor`

### department_type
部门类型：`clinical`, `medical_technical`, `administrative`, `logistics`

### diagnosis_type
诊断类型：`primary`, `secondary`, `admission`, `discharge`

### nursing_level
护理等级：`self_care`, `grade_1`, `grade_2`, `grade_3`, `critical`

### specimen_status
样本状态：`collected`, `received`, `processing`, `completed`, `rejected`

### charge_category
收费类别：`registration`, `examination`, `laboratory`, `medicine`, `treatment`, `surgery`, `hospitalization`, `nursing`, `other`

### marital_status
婚姻状态：`single`, `married`, `divorced`, `widowed`

### medicine_type
药品类型：`western`, `chinese`, `herbal`, `consumable`, `equipment`

## 表结构 (Tables)

### patients
患者表
- `id`: integer (PRIMARY KEY)
- `patient_no`: character varying (NOT NULL) - 患者编号
- `name`: character varying (NOT NULL) - 姓名
- `gender`: gender_enum - 性别
- `birth_date`: date - 出生日期
- `blood_type`: blood_type_enum - 血型
- `phone`: character varying - 电话
- `email`: character varying - 邮箱
- `address`: text - 地址
- `marital_status`: marital_status - 婚姻状态
- `emergency_contact`: character varying - 紧急联系人
- `emergency_phone`: character varying - 紧急电话
- `insurance_no`: character varying - 医保号

### patient_records
患者档案表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (NOT NULL, 外键 patients.id) - 患者ID
- `record_date`: date - 档案日期
- `height`: numeric - 身高
- `weight`: numeric - 体重
- `blood_pressure`: character varying - 血压
- `heart_rate`: integer - 心率
- `allergies`: text - 过敏史
- `notes`: text - 备注

### medical_history
病史表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `disease_name`: character varying - 疾病名称
- `diagnosis_date`: date - 诊断日期
- `treatment`: text - 治疗方案
- `status`: character varying - 状态

### family_history
家族病史表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `relationship`: character varying - 关系
- `condition`: character varying - 病症
- `notes`: text - 备注

### allergies
过敏史表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `allergen`: character varying (NOT NULL) - 过敏源
- `reaction`: text - 反应
- `severity`: severity_level - 严重程度
- `discovered_date`: date - 发现日期

### departments
部门表
- `id`: integer (PRIMARY KEY)
- `name`: character varying (NOT NULL) - 部门名称
- `department_type`: department_type - 部门类型
- `location`: character varying - 位置
- `phone`: character varying - 电话

### staff
员工表
- `id`: integer (PRIMARY KEY)
- `staff_no`: character varying (NOT NULL) - 员工编号
- `name`: character varying (NOT NULL) - 姓名
- `staff_type`: staff_type - 员工类型
- `doctor_title`: doctor_title - 医生头衔
- `department_id`: integer - 部门ID (外键 departments.id)
- `phone`: character varying - 电话
- `email`: character varying - 邮箱
- `specialty`: character varying - 专长
- `status`: character varying - 状态

### registrations
挂号表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `department_id`: integer (外键 departments.id) - 部门ID
- `doctor_id`: integer - 医生ID (外键 staff.id)
- `registration_time`: timestamp without time zone - 挂号时间
- `registration_type`: patient_type - 挂号类型
- `status`: registration_status - 挂号状态
- `priority`: priority_level - 优先级

### appointments
预约表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `doctor_id`: integer (外键 staff.id) - 医生ID
- `appointment_date`: date - 预约日期
- `start_time`: time without time zone - 开始时间
- `end_time`: time without time zone - 结束时间
- `appointment_type`: character varying - 预约类型
- `status`: appointment_status - 预约状态

### outpatient_visits
门诊就诊表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `doctor_id`: integer (外键 staff.id) - 医生ID
- `registration_id`: integer - 挂号ID (外键 registrations.id)
- `visit_date`: date - 就诊日期
- `chief_complaint`: text - 主诉
- `diagnosis`: text - 诊断
- `treatment`: text - 治疗

### diagnoses
诊断表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `visit_id`: integer - 就诊ID (外键 outpatient_visits.id)
- `diagnosis_code`: character varying - 诊断编码
- `diagnosis_name`: character varying - 诊断名称
- `diagnosis_type`: diagnosis_type - 诊断类型
- `diagnosis_date`: date - 诊断日期

### progress_notes
病程记录表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `visit_id`: integer - 就诊ID (外键 outpatient_visits.id)
- `author_id`: integer - 记录人ID (外键 staff.id)
- `note_date`: date - 记录日期
- `note_type`: character varying - 记录类型
- `content`: text - 内容

### wards
病房表
- `id`: integer (PRIMARY KEY)
- `ward_no`: character varying (NOT NULL) - 病房号
- `ward_type`: character varying - 病房类型
- `department_id`: integer - 部门ID (外键 departments.id)
- `floor`: integer - 楼层
- `capacity`: integer - 床位容量

### beds
床位表
- `id`: integer (PRIMARY KEY)
- `bed_no`: character varying (NOT NULL) - 床位号
- `ward_id`: integer (外键 wards.id) - 病房ID
- `bed_status`: bed_status - 床位状态
- `bed_type`: character varying - 床位类型

### admissions
入院表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer ( (外键 patients.id) - 患者ID
- `ward_id`: integer (外键 wards.id) - 病房ID
- `bed_id`: integer - 床位ID (外键 beds.id)
- `admission_date`: date - 入院日期
- `discharge_date`: date - 出院日期
- `admission_type`: character varying - 入院类型
- `diagnosis`: text - 诊断
- `status`: character varying - 状态

### medical_orders
医嘱表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `admission_id`: integer - 入院ID (外键 admissions.id)
- `order_type`: character varying - 医嘱类型
- `order_content`: text - 医嘱内容
- `order_date`: date - 医嘱日期
- `frequency`: character varying - 频率
- `start_time`: timestamp without time zone - 开始时间
- `stop_time`: timestamp without time zone - 停止时间
- `status`: character varying - 状态
- `ordering_doctor_id`: integer - 开具医生ID (外键 staff.id)

### nursing_records
护理记录表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `admission_id`: integer - 入院ID (外键 admissions.id)
- `nurse_id`: integer - 护士ID (外键 staff.id)
- `record_date`: date - 记录日期
- `record_time`: time without time zone - 记录时间
- `nursing_level`: nursing_level - 护理等级
- `content`: text - 内容

### nursing_plans
护理计划表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `admission_id`: integer - 入院ID (外键 admissions.id)
- `plan_date`: date - 计划日期
- `nursing_diagnosis`: text - 护理诊断
- `goals`: text - 护理目标
- `interventions`: text - 护理措施

### prescriptions
处方表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `doctor_id`: integer (外键 staff.id) - 医生ID
- `visit_id`: integer - 就诊ID (外键 outpatient_visits.id)
- `prescription_date`: date - 处方日期
- `prescription_no`: character varying - 处方号
- `status`: prescription_status - 处方状态

### prescription_items
处方明细表
- `id`: integer (PRIMARY KEY)
- `prescription_id`: integer (外键 prescriptions.id) - 处方ID
- `medicine_id`: integer - 药品ID (外键 medicines.id)
- `dosage`: character varying - 剂量
- `frequency`: character varying - 频率
- `duration`: integer - 持续天数
- `quantity`: integer - 数量
- `instructions`: text - 用法说明

### medicines
药品表
- `id`: integer (PRIMARY KEY)
- `medicine_no`: character varying (NOT NULL) - 药品编号
- `name`: character varying (NOT NULL) - 药品名称
- `generic_name`: character varying - 通用名
- `medicine_type`: medicine_type - 药品类型
- `dosage_form`: character varying - 剂型
- `strength`: character varying - 规格
- `manufacturer`: character varying - 生产厂家

### medicine_inventory
药品库存表
- `id`: integer (PRIMARY KEY)
- `medicine_id`: integer (外键 medicines.id) - 药品ID
- `batch_no`: character varying - 批号
- `expiry_date`: date - 过期日期
- `quantity`: integer - 数量
- `unit_price`: numeric - 单价
- `location`: character varying - 存放位置

### examination_requests
检查申请表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `doctor_id`: integer (外键 staff.id) - 医生ID
- `visit_id`: integer - 就诊ID (外键 outpatient_visits.id)
- `examination_type`: character varying - 检查类型
- `request_date`: date - 申请日期
- `priority`: priority_level - 优先级
- `status`: exam_status - 检查状态

### examination_reports
检查报告表
- `id`: integer (PRIMARY KEY)
- `request_id`: integer (外键 examination_requests.id) - 申请ID
- `report_date`: date - 报告日期
- `findings`: text - 检查发现
- `impression`: text - 印象诊断
- `reporting_technician_id`: integer - 报告技师ID (外键 staff.id)

### examination_items
检查项目表
- `id`: integer (PRIMARY KEY)
- `report_id`: integer - 报告ID (外键 examination_reports.id)
- `item_name`: character varying - 项目名称
- `item_value`: character varying - 检查值
- `reference_range`: character varying - 参考范围
- `unit`: character varying - 单位
- `is_abnormal`: boolean - 是否异常

### laboratory_tests
检验项目表
- `id`: integer (PRIMARY KEY)
- `test_code`: character varying (NOT NULL) - 检验代码
- `test_name`: character varying (NOT NULL) - 检验名称
- `category`: character varying - 类别
- `reference_range`: character varying - 参考范围
- `unit`: character varying - 单位

### laboratory_requests
检验申请表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `doctor_id`: integer (外键 staff.id) - 医生ID
- `visit_id`: integer - 就诊ID (外键 outpatient_visits.id)
- `test_id`: integer - 检验项目ID (外键 laboratory_tests.id)
- `request_date`: date - 申请日期
- `priority`: priority_level - 优先级
- `status`: specimen_status - 状态

### laboratory_results
检验结果表
- `id`: integer (PRIMARY KEY)
- `request_id`: integer (外键 laboratory_requests.id) - 申请ID
- `result_date`: date - 结果日期
- `result_value`: character varying - 检验值
- `is_abnormal`: boolean - 是否异常
- `notes`: text - 备注
- `reporting_staff_id`: integer - 报告人ID (外键 staff.id)

### operating_rooms
手术室表
- `id`: integer (PRIMARY KEY)
- `room_no`: character varying (NOT NULL) - 手术室号
- `location`: character varying - 位置
- `capacity`: integer - 容量
- `status`: character varying - 状态
- `equipment`: text - 设备

### surgery_requests
手术申请表
- `id`: integer (PRIMARY KEY)
- ``patient_id`: integer (外键 patients.id) - 患者ID
- `doctor_id`: integer (外键 staff.id) - 主刀医生ID
- `surgery_type`: character varying - 手术类型
- `requested_date`: date - 申请日期
- `planned_date`: date - 计划日期
- `priority`: priority_level - 优先级
- `status`: surgery_status - 手术状态
- `operating_room_id`: integer - 手术室ID (外键 operating_rooms.id)
- `estimated_duration`: integer - 预计时长(分钟)
- `anesthesia_type`: anesthesia_type - 麻醉类型

### surgery_records
手术记录表
- `id`: integer (PRIMARY KEY)
- `request_id`: integer (外键 surgery_requests.id) - 申请ID
- `start_time`: timestamp without time zone - 开始时间
- `end_time`: timestamp without time zone - 结束时间
- `procedure_details`: text - 手术过程
- `complications`: text - 并发症
- `notes`: text - 备注

### emergency_triage
急诊分诊表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `triage_time`: timestamp without time zone - 分诊时间
- `priority`: priority_level - 优先级
- `chief_complaint`: text - 主诉
- `vital_signs`: text - 生命体征
- `triage_nurse_id`: integer - 分诊护士ID (外键 staff.id)

### emergency_visits
急诊就诊表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `triage_id`: integer - 分诊ID (外键 emergency_triage.id)
- `doctor_id`: integer - 医生ID (外键 staff.id)
- `visit_time`: timestamp without time zone - 就诊时间
- `diagnosis`: text - 诊断
- `treatment`: text - 治疗
- `disposition`: character varying - 处置

### schedules
排班表
- `id`: integer (PRIMARY KEY)
- `staff_id`: integer (外键 staff.id) - 员工ID
- `schedule_date`: date - 排班日期
- `shift_type`: character varying - 班次类型
- `start_time`: time without time zone - 开始时间
- `end_time`: time without time zone - 结束时间
- `department_id`: integer - 部门ID (外键 departments.id)

### bills
账单表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `visit_id`: integer - 就诊ID (外键 outpatient_visits.id)
- `admission_id`: integer - 入院ID (外键 admissions.id)
- `bill_date`: date - 账单日期
- `total_amount`: numeric - 总金额
- `paid_amount`: numeric - 已付金额
- `status`: character varying - 状态

### charges
收费明细表
- `id`: integer (PRIMARY KEY)
- `bill_id`: integer - 账单ID (外键 bills.id)
- `charge_date`: date - 收费日期
- `charge_amount`: numeric - 收费金额
- `category`: charge_category - 收费类别

### charge_items
收费项目表
- `id`: integer (PRIMARY KEY)
- `charge_id`: integer - 收费ID (外键 charges.id)
- `item_name`: character varying - 项目名称
- `item_code`: character varying - 项目代码
- `quantity`: integer - 数量
- `unit_price`: numeric - 单价
- `subtotal`: numeric - 小计

### payments
支付表
- `id`: integer (PRIMARY KEY)
- `bill_id`: integer - 账单ID (外键 bills.id)
- `payment_date`: date - 支付日期
- `payment_amount`: numeric - 支付金额
- `payment_method`: payment_method - 支付方式
- `payment_no`: character varying - 支付号

### deposits
预交金表
- `id`: integer (PRIMARY KEY)
- `patient_id`: integer (外键 patients.id) - 患者ID
- `deposit_date`: date - 预交日期
- `deposit_amount`: numeric - 预交金额
- `balance`: numeric - 余额
- `payment_method`: payment_method - 支付方式

## 视图 (Views)

### v_patient_basic_info
患者基本信息视图

### v_today_registrations
今日挂号视图

### v_current_inpatients
在院患者视图

### v_bed_utilization
床位使用统计视图

### v_doctor_schedule_today
医生今日排班视图

### v_pending_examinations
待检查项目视图

### v_pending_laboratory_tests
待检验项目视图

### v_pending_prescriptions
待配药处方视图

### v_surgery_schedule
手术安排视图

### v_patient_diagnosis_history
患者诊断历史视图

### v_patient_charges_summary
患者费用汇总视图

### v_department_statistics
部门统计视图

### v_emergency_statistics
急诊统计视图

### v_medicine_stock_alert
药品库存预警视图

## 核心业务流程

1. **挂号流程**: registrations -> appointments -> outpatient_visits
2. **住院流程**: admissions -> medical_orders -> nursing_records -> nursing_plans
3. **用药流程**: prescriptions -> prescription_items -> medicines -> medicine_inventory
4. **检查检验流程**: examination_requests -> examination_reports, laboratory_requests -> laboratory_results
5. **手术流程**: surgery_requests -> surgery_records
6. **急诊流程**: emergency_triage -> emergency_visits
7. **收费流程**: bills -> charges -> charge_items -> payments
