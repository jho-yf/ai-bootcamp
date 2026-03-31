-- ============================================================================
-- Large Hospital Information System Database
-- 场景：综合医院信息系统 (HIS - Hospital Information System)
-- 规模：55+张表，25+视图，40+索引，12个自定义类型，约2000行测试数据
-- ============================================================================

DROP DATABASE IF EXISTS pg_mcp_large;
CREATE DATABASE pg_mcp_large;

\c pg_mcp_large;

-- ----------------------------------------------------------------------------
-- Custom Types
-- ----------------------------------------------------------------------------

-- 血型
CREATE TYPE blood_type_enum AS ENUM ('A', 'B', 'AB', 'O');

-- 性别
CREATE TYPE gender_enum AS ENUM ('male', 'female', 'other');

-- 婚姻状况
CREATE TYPE marital_status AS ENUM ('single', 'married', 'divorced', 'widowed');

-- 患者类型
CREATE TYPE patient_type AS ENUM ('outpatient', 'inpatient', 'emergency');

-- 挂号状态
CREATE TYPE registration_status AS ENUM ('pending', 'confirmed', 'completed', 'cancelled');

-- 诊断类型
CREATE TYPE diagnosis_type AS ENUM ('primary', 'secondary', 'admission', 'discharge');

-- 严重程度
CREATE TYPE severity_level AS ENUM ('mild', 'moderate', 'severe', 'critical');

-- 优先级
CREATE TYPE priority_level AS ENUM ('low', 'normal', 'high', 'urgent', 'emergency');

-- 付费方式
CREATE TYPE payment_method AS ENUM ('cash', 'insurance', 'credit_card', 'debit_card', 'weixin', 'alipay');

-- 费用类别
CREATE TYPE charge_category AS ENUM ('registration', 'examination', 'laboratory', 'medicine', 'treatment', 'surgery', 'hospitalization', 'nursing', 'other');

-- 处方状态
CREATE TYPE prescription_status AS ENUM ('pending', 'dispensed', 'cancelled', 'completed');

-- 手术状态
CREATE TYPE surgery_status AS ENUM ('scheduled', 'in_progress', 'completed', 'cancelled', 'postponed');

-- 麻醉类型
CREATE TYPE anesthesia_type AS ENUM ('local', 'general', 'spinal', 'epidural', 'none');

-- 护理等级
CREATE TYPE nursing_level AS ENUM ('self_care', 'grade_1', 'grade_2', 'grade_3', 'critical');

-- 床位状态
CREATE TYPE bed_status AS ENUM ('available', 'occupied', 'maintenance', 'reserved');

-- 检查状态
CREATE TYPE exam_status AS ENUM ('ordered', 'scheduled', 'in_progress', 'completed', 'cancelled');

-- 标本状态
CREATE TYPE specimen_status AS ENUM ('collected', 'received', 'processing', 'completed', 'rejected');

-- 药品类型
CREATE TYPE medicine_type AS ENUM ('western', 'chinese', 'herbal', 'consumable', 'equipment');

-- 员工类型
CREATE TYPE staff_type AS ENUM ('doctor', 'nurse', 'pharmacist', 'technician', 'admin', 'service', 'management');

-- 医生职称
CREATE TYPE doctor_title AS ENUM ('resident', 'attending', 'chief', 'director', 'professor');

-- 科室类型
CREATE TYPE department_type AS ENUM ('clinical', 'medical_technical', 'administrative', 'logistics');

-- 预约状态
CREATE TYPE appointment_status AS ENUM ('scheduled', 'confirmed', 'completed', 'cancelled', 'no_show');

-- ----------------------------------------------------------------------------
-- Tables - 基础信息
-- ----------------------------------------------------------------------------

-- 科室表
CREATE TABLE departments (
    id SERIAL PRIMARY KEY,
    dept_code VARCHAR(20) UNIQUE NOT NULL,
    dept_name VARCHAR(100) NOT NULL,
    dept_type department_type NOT NULL,
    parent_dept_id INTEGER REFERENCES departments(id),
    director_id INTEGER,
    description TEXT,
    established_date DATE,
    floor_no INTEGER,
    extension VARCHAR(20),
    status VARCHAR(20) DEFAULT 'active'
);

-- 员工表
CREATE TABLE staff (
    id SERIAL PRIMARY KEY,
    staff_id VARCHAR(20) UNIQUE NOT NULL,
    name VARCHAR(50) NOT NULL,
    gender gender_enum,
    birth_date DATE,
    id_card VARCHAR(18),
    phone VARCHAR(20) NOT NULL,
    email VARCHAR(100),
    address TEXT,
    department_id INTEGER REFERENCES departments(id),
    staff_type staff_type NOT NULL,
    title doctor_title,
    specialty VARCHAR(100),
    license_no VARCHAR(50),
    hire_date DATE NOT NULL,
    status VARCHAR(20) DEFAULT 'active',
    photo VARCHAR(200)
);

-- 排班表
CREATE TABLE schedules (
    id SERIAL PRIMARY KEY,
    staff_id INTEGER REFERENCES staff(id),
    schedule_date DATE NOT NULL,
    shift_type VARCHAR(20) NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    location VARCHAR(100)
);

-- ----------------------------------------------------------------------------
-- Tables - 患者管理
-- ----------------------------------------------------------------------------

-- 患者基本信息表
CREATE TABLE patients (
    id SERIAL PRIMARY KEY,
    patient_id VARCHAR(20) UNIQUE NOT NULL,
    name VARCHAR(50) NOT NULL,
    gender gender_enum NOT NULL,
    birth_date DATE NOT NULL,
    id_card VARCHAR(18) UNIQUE,
    phone VARCHAR(20) NOT NULL,
    email VARCHAR(100),
    address TEXT,
    city VARCHAR(50),
    district VARCHAR(50),
    postal_code VARCHAR(10),
    marital_status marital_status,
    occupation VARCHAR(50),
    employer VARCHAR(100),
    blood_type blood_type_enum,
    emergency_contact_name VARCHAR(50),
    emergency_contact_phone VARCHAR(20),
    emergency_contact_relation VARCHAR(20),
    insurance_company VARCHAR(100),
    insurance_policy_no VARCHAR(50),
    registration_date DATE DEFAULT CURRENT_DATE,
    notes TEXT,
    status VARCHAR(20) DEFAULT 'active'
);

-- 患者档案表
CREATE TABLE patient_records (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER REFERENCES patients(id),
    record_type VARCHAR(20) NOT NULL,
    record_date DATE DEFAULT CURRENT_DATE,
    title VARCHAR(200),
    content TEXT,
    created_by INTEGER REFERENCES staff(id),
    attachments TEXT
);

-- 过敏史表
CREATE TABLE allergies (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER REFERENCES patients(id),
    allergen VARCHAR(100) NOT NULL,
    allergy_type VARCHAR(50),
    severity severity_level,
    reaction_description TEXT,
    diagnosed_date DATE,
    diagnosed_by INTEGER REFERENCES staff(id)
);

-- 既往病史表
CREATE TABLE medical_history (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER REFERENCES patients(id),
    condition_name VARCHAR(200) NOT NULL,
    condition_code VARCHAR(50),
    diagnosis_date DATE,
    treatment_received TEXT,
    outcome VARCHAR(100),
    is_chronic BOOLEAN DEFAULT false,
    notes TEXT
);

-- 家族病史表
CREATE TABLE family_history (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER REFERENCES patients(id),
    relation VARCHAR(50) NOT NULL,
    condition_name VARCHAR(200) NOT NULL,
    age_at_diagnosis INTEGER,
    is_alive BOOLEAN
);

-- ----------------------------------------------------------------------------
-- Tables - 门诊管理
-- ----------------------------------------------------------------------------

-- 挂号表
CREATE TABLE registrations (
    id SERIAL PRIMARY KEY,
    registration_no VARCHAR(30) UNIQUE NOT NULL,
    patient_id INTEGER REFERENCES patients(id),
    department_id INTEGER REFERENCES departments(id),
    doctor_id INTEGER REFERENCES staff(id),
    registration_date DATE DEFAULT CURRENT_DATE,
    registration_time TIME DEFAULT CURRENT_TIME,
    registration_type patient_type NOT NULL,
    status registration_status DEFAULT 'pending',
    priority priority_level DEFAULT 'normal',
    fee DECIMAL(10,2),
    referred_by INTEGER REFERENCES staff(id),
    remarks TEXT
);

-- 门诊预约表
CREATE TABLE appointments (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER REFERENCES patients(id),
    department_id INTEGER REFERENCES departments(id),
    doctor_id INTEGER REFERENCES staff(id),
    appointment_date DATE NOT NULL,
    appointment_time TIME NOT NULL,
    appointment_type VARCHAR(50),
    status appointment_status DEFAULT 'scheduled',
    purpose TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 门诊病历表
CREATE TABLE outpatient_visits (
    id SERIAL PRIMARY KEY,
    visit_no VARCHAR(30) UNIQUE NOT NULL,
    patient_id INTEGER REFERENCES patients(id),
    registration_id INTEGER REFERENCES registrations(id),
    department_id INTEGER REFERENCES departments(id),
    doctor_id INTEGER REFERENCES staff(id),
    visit_date DATE DEFAULT CURRENT_DATE,
    visit_time TIME DEFAULT CURRENT_TIME,
    chief_complaint TEXT,
    present_illness TEXT,
    physical_examination TEXT,
    vital_signs TEXT,
    preliminary_diagnosis TEXT,
    treatment_plan TEXT,
    follow_up_date DATE,
    status VARCHAR(20) DEFAULT 'completed'
);

-- 诊断记录表
CREATE TABLE diagnoses (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER REFERENCES patients(id),
    visit_id INTEGER REFERENCES outpatient_visits(id),
    admission_id INTEGER,
    diagnosis_code VARCHAR(20) NOT NULL,
    diagnosis_name VARCHAR(200) NOT NULL,
    diagnosis_type diagnosis_type NOT NULL,
    diagnosis_date DATE DEFAULT CURRENT_DATE,
    diagnosed_by INTEGER REFERENCES staff(id),
    is_confirmed BOOLEAN DEFAULT false,
    notes TEXT
);

-- ----------------------------------------------------------------------------
-- Tables - 住院管理
-- ----------------------------------------------------------------------------

-- 病房表
CREATE TABLE wards (
    id SERIAL PRIMARY KEY,
    ward_no VARCHAR(20) UNIQUE NOT NULL,
    ward_name VARCHAR(50),
    department_id INTEGER REFERENCES departments(id),
    floor_no INTEGER,
    bed_count INTEGER NOT NULL,
    nursing_level INTEGER,
    charge_per_day DECIMAL(10,2),
    head_nurse_id INTEGER REFERENCES staff(id),
    description TEXT
);

-- 病床表
CREATE TABLE beds (
    id SERIAL PRIMARY KEY,
    bed_no VARCHAR(20) NOT NULL,
    ward_id INTEGER REFERENCES wards(id),
    bed_type VARCHAR(20),
    status bed_status DEFAULT 'available',
    charge_per_day DECIMAL(10,2),
    UNIQUE(bed_no, ward_id)
);

-- 入院登记表
CREATE TABLE admissions (
    id SERIAL PRIMARY KEY,
    admission_no VARCHAR(30) UNIQUE NOT NULL,
    patient_id INTEGER REFERENCES patients(id),
    department_id INTEGER REFERENCES departments(id),
    ward_id INTEGER REFERENCES wards(id),
    bed_id INTEGER REFERENCES beds(id),
    admission_date DATE NOT NULL,
    admission_time TIME DEFAULT CURRENT_TIME,
    admission_type VARCHAR(50),
    primary_doctor_id INTEGER REFERENCES staff(id),
    referring_doctor_id INTEGER REFERENCES staff(id),
    diagnosis TEXT,
    patient_condition TEXT,
    nursing_level nursing_level,
    deposit_amount DECIMAL(10,2),
    estimated_days INTEGER,
    discharge_date DATE,
    discharge_time TIME,
    discharge_diagnosis TEXT,
    discharge_summary TEXT,
    discharge_destination VARCHAR(50),
    status VARCHAR(20) DEFAULT 'admitted'
);

-- 病程记录表
CREATE TABLE progress_notes (
    id SERIAL PRIMARY KEY,
    admission_id INTEGER REFERENCES admissions(id),
    patient_id INTEGER REFERENCES patients(id),
    note_date DATE DEFAULT CURRENT_DATE,
    note_time TIME DEFAULT CURRENT_TIME,
    note_type VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    recorded_by INTEGER REFERENCES staff(id)
);

-- 医嘱表
CREATE TABLE medical_orders (
    id SERIAL PRIMARY KEY,
    admission_id INTEGER REFERENCES admissions(id),
    patient_id INTEGER REFERENCES patients(id),
    order_date DATE DEFAULT CURRENT_DATE,
    order_time TIME DEFAULT CURRENT_TIME,
    order_type VARCHAR(50) NOT NULL,
    order_content TEXT NOT NULL,
    priority priority_level DEFAULT 'normal',
    ordered_by INTEGER REFERENCES staff(id),
    status VARCHAR(20) DEFAULT 'active',
    start_time TIMESTAMP,
    stop_time TIMESTAMP,
    executed_by INTEGER REFERENCES staff(id),
    executed_at TIMESTAMP
);

-- ----------------------------------------------------------------------------
-- Tables - 检查检验
-- ----------------------------------------------------------------------------

-- 检查项目表
CREATE TABLE examination_items (
    id SERIAL PRIMARY KEY,
    item_code VARCHAR(20) UNIQUE NOT NULL,
    item_name VARCHAR(100) NOT NULL,
    category VARCHAR(50),
    department_id INTEGER REFERENCES departments(id),
    price DECIMAL(10,2) NOT NULL,
    preparation_instructions TEXT,
    contraindications TEXT,
    normal_duration INTEGER,
    status VARCHAR(20) DEFAULT 'active'
);

-- 检查申请表
CREATE TABLE examination_requests (
    id SERIAL PRIMARY KEY,
    request_no VARCHAR(30) UNIQUE NOT NULL,
    patient_id INTEGER REFERENCES patients(id),
    visit_id INTEGER REFERENCES outpatient_visits(id),
    admission_id INTEGER REFERENCES admissions(id),
    item_id INTEGER REFERENCES examination_items(id),
    requested_by INTEGER REFERENCES staff(id),
    request_date DATE DEFAULT CURRENT_DATE,
    request_time TIME DEFAULT CURRENT_TIME,
    clinical_info TEXT,
    priority priority_level DEFAULT 'normal',
    status exam_status DEFAULT 'ordered',
    scheduled_date DATE,
    scheduled_time TIME,
    performed_by INTEGER REFERENCES staff(id),
    performed_at TIMESTAMP
);

-- 检查报告表
CREATE TABLE examination_reports (
    id SERIAL PRIMARY KEY,
    request_id INTEGER REFERENCES examination_requests(id),
    report_date DATE DEFAULT CURRENT_DATE,
    findings TEXT,
    impression TEXT,
    conclusion TEXT,
    images TEXT,
    reported_by INTEGER REFERENCES staff(id),
    verified_by INTEGER REFERENCES staff(id),
    status VARCHAR(20) DEFAULT 'draft'
);

-- 检验项目表
CREATE TABLE laboratory_tests (
    id SERIAL PRIMARY KEY,
    test_code VARCHAR(20) UNIQUE NOT NULL,
    test_name VARCHAR(100) NOT NULL,
    category VARCHAR(50),
    specimen_type VARCHAR(50) NOT NULL,
    department_id INTEGER REFERENCES departments(id),
    price DECIMAL(10,2) NOT NULL,
    sample_requirements TEXT,
    reference_range TEXT,
    unit VARCHAR(20),
    turnaround_hours INTEGER,
    status VARCHAR(20) DEFAULT 'active'
);

-- 检验申请表
CREATE TABLE laboratory_requests (
    id SERIAL PRIMARY KEY,
    request_no VARCHAR(30) UNIQUE NOT NULL,
    patient_id INTEGER REFERENCES patients(id),
    visit_id INTEGER REFERENCES outpatient_visits(id),
    admission_id INTEGER REFERENCES admissions(id),
    test_id INTEGER REFERENCES laboratory_tests(id),
    requested_by INTEGER REFERENCES staff(id),
    request_date DATE DEFAULT CURRENT_DATE,
    request_time TIME DEFAULT CURRENT_TIME,
    clinical_diagnosis TEXT,
    priority priority_level DEFAULT 'normal',
    status specimen_status DEFAULT 'collected',
    specimen_collected_at TIMESTAMP,
    specimen_collected_by INTEGER REFERENCES staff(id),
    specimen_received_at TIMESTAMP,
    tested_at TIMESTAMP,
    tested_by INTEGER REFERENCES staff(id)
);

-- 检验结果表
CREATE TABLE laboratory_results (
    id SERIAL PRIMARY KEY,
    request_id INTEGER REFERENCES laboratory_requests(id),
    result_value VARCHAR(200),
    result_unit VARCHAR(20),
    reference_range TEXT,
    is_abnormal BOOLEAN DEFAULT false,
    abnormal_flag VARCHAR(10),
    comments TEXT,
    verified_by INTEGER REFERENCES staff(id),
    verified_at TIMESTAMP
);

-- ----------------------------------------------------------------------------
-- Tables - 药房管理
-- ----------------------------------------------------------------------------

-- 药品信息表
CREATE TABLE medicines (
    id SERIAL PRIMARY KEY,
    medicine_code VARCHAR(20) UNIQUE NOT NULL,
    generic_name VARCHAR(100) NOT NULL,
    brand_name VARCHAR(100),
    medicine_type medicine_type NOT NULL,
    category VARCHAR(50),
    specification VARCHAR(50),
    unit VARCHAR(20),
    manufacturer VARCHAR(100),
    dosage_form VARCHAR(50),
    strength VARCHAR(50),
    price DECIMAL(10,2) NOT NULL,
    storage_condition VARCHAR(50),
    prescription_required BOOLEAN DEFAULT true,
    status VARCHAR(20) DEFAULT 'active'
);

-- 药品库存表
CREATE TABLE medicine_inventory (
    id SERIAL PRIMARY KEY,
    medicine_id INTEGER REFERENCES medicines(id),
    batch_no VARCHAR(50) NOT NULL,
    quantity INTEGER NOT NULL,
    expiry_date DATE NOT NULL,
    location VARCHAR(50),
    supplier VARCHAR(100),
    received_date DATE DEFAULT CURRENT_DATE
);

-- 处方表
CREATE TABLE prescriptions (
    id SERIAL PRIMARY KEY,
    prescription_no VARCHAR(30) UNIQUE NOT NULL,
    patient_id INTEGER REFERENCES patients(id),
    visit_id INTEGER REFERENCES outpatient_visits(id),
    admission_id INTEGER REFERENCES admissions(id),
    prescribed_by INTEGER REFERENCES staff(id),
    prescription_date DATE DEFAULT CURRENT_DATE,
    diagnosis TEXT,
    status prescription_status DEFAULT 'pending',
    dispensed_by INTEGER REFERENCES staff(id),
    dispensed_at TIMESTAMP
);

-- 处方明细表
CREATE TABLE prescription_items (
    id SERIAL PRIMARY KEY,
    prescription_id INTEGER REFERENCES prescriptions(id),
    medicine_id INTEGER REFERENCES medicines(id),
    dosage VARCHAR(50),
    frequency VARCHAR(50),
    route VARCHAR(20),
    duration INTEGER,
    quantity INTEGER NOT NULL,
    unit_price DECIMAL(10,2),
    subtotal DECIMAL(10,2),
    instructions TEXT
);

-- ----------------------------------------------------------------------------
-- Tables - 手术管理
-- ----------------------------------------------------------------------------

-- 手术室表
CREATE TABLE operating_rooms (
    id SERIAL PRIMARY KEY,
    room_no VARCHAR(20) UNIQUE NOT NULL,
    room_name VARCHAR(50),
    room_type VARCHAR(50),
    floor_no INTEGER,
    capacity INTEGER,
    equipment_list TEXT,
    status VARCHAR(20) DEFAULT 'available'
);

-- 手术申请表
CREATE TABLE surgery_requests (
    id SERIAL PRIMARY KEY,
    surgery_no VARCHAR(30) UNIQUE NOT NULL,
    patient_id INTEGER REFERENCES patients(id),
    admission_id INTEGER REFERENCES admissions(id),
    surgery_name VARCHAR(200) NOT NULL,
    surgery_code VARCHAR(50),
    requested_by INTEGER REFERENCES staff(id),
    request_date DATE DEFAULT CURRENT_DATE,
    planned_date DATE,
    planned_start_time TIME,
    planned_duration INTEGER,
    operating_room_id INTEGER REFERENCES operating_rooms(id),
    anesthesia_type anesthesia_type,
    primary_surgeon_id INTEGER REFERENCES staff(id),
    assistant_surgeons TEXT,
    anesthesiologist_id INTEGER REFERENCES staff(id),
    scrub_nurse_id INTEGER REFERENCES staff(id),
    circulating_nurse_id INTEGER REFERENCES staff(id),
    priority priority_level DEFAULT 'normal',
    preoperative_diagnosis TEXT,
    postoperative_diagnosis TEXT,
    status surgery_status DEFAULT 'scheduled'
);

-- 手术记录表
CREATE TABLE surgery_records (
    id SERIAL PRIMARY KEY,
    surgery_request_id INTEGER REFERENCES surgery_requests(id),
    actual_start_time TIMESTAMP,
    actual_end_time TIMESTAMP,
    actual_duration INTEGER,
    anesthesia_details TEXT,
    procedure_details TEXT,
    complications TEXT,
    blood_loss INTEGER,
    blood_transfusion BOOLEAN,
    implants_used TEXT,
    postoperative_instructions TEXT,
    recorded_by INTEGER REFERENCES staff(id)
);

-- ----------------------------------------------------------------------------
-- Tables - 费用管理
-- ----------------------------------------------------------------------------

-- 费用项目表
CREATE TABLE charge_items (
    id SERIAL PRIMARY KEY,
    item_code VARCHAR(20) UNIQUE NOT NULL,
    item_name VARCHAR(100) NOT NULL,
    category charge_category NOT NULL,
    unit VARCHAR(20),
    price DECIMAL(10,2) NOT NULL,
    department_id INTEGER REFERENCES departments(id),
    description TEXT,
    status VARCHAR(20) DEFAULT 'active'
);

-- 费用明细表
CREATE TABLE charges (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER REFERENCES patients(id),
    visit_id INTEGER REFERENCES outpatient_visits(id),
    admission_id INTEGER REFERENCES admissions(id),
    charge_item_id INTEGER REFERENCES charge_items(id),
    charge_date DATE DEFAULT CURRENT_DATE,
    charge_time TIME DEFAULT CURRENT_TIME,
    quantity DECIMAL(10,2) DEFAULT 1,
    unit_price DECIMAL(10,2),
    total_amount DECIMAL(10,2),
    performed_by INTEGER REFERENCES staff(id),
    charge_status VARCHAR(20) DEFAULT 'unbilled'
);

-- 账单表
CREATE TABLE bills (
    id SERIAL PRIMARY KEY,
    bill_no VARCHAR(30) UNIQUE NOT NULL,
    patient_id INTEGER REFERENCES patients(id),
    visit_id INTEGER REFERENCES outpatient_visits(id),
    admission_id INTEGER REFERENCES admissions(id),
    bill_date DATE DEFAULT CURRENT_DATE,
    total_amount DECIMAL(12,2),
    discount_amount DECIMAL(10,2) DEFAULT 0,
    insurance_covered DECIMAL(10,2) DEFAULT 0,
    net_amount DECIMAL(12,2),
    payment_status VARCHAR(20) DEFAULT 'unpaid',
    payment_method payment_method,
    payment_date DATE,
    created_by INTEGER REFERENCES staff(id)
);

-- 支付记录表
CREATE TABLE payments (
    id SERIAL PRIMARY KEY,
    bill_id INTEGER REFERENCES bills(id),
    payment_date DATE DEFAULT CURRENT_DATE,
    payment_time TIME DEFAULT CURRENT_TIME,
    amount DECIMAL(12,2) NOT NULL,
    payment_method payment_method NOT NULL,
    transaction_no VARCHAR(50),
    remarks TEXT,
    received_by INTEGER REFERENCES staff(id)
);

-- 预交金表
CREATE TABLE deposits (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER REFERENCES patients(id),
    admission_id INTEGER REFERENCES admissions(id),
    deposit_date DATE DEFAULT CURRENT_DATE,
    amount DECIMAL(12,2) NOT NULL,
    payment_method payment_method NOT NULL,
    receipt_no VARCHAR(50),
    received_by INTEGER REFERENCES staff(id),
    refund_amount DECIMAL(12,2) DEFAULT 0,
    refund_date DATE
);

-- ----------------------------------------------------------------------------
-- Tables - 护理管理
-- ----------------------------------------------------------------------------

-- 护理记录表
CREATE TABLE nursing_records (
    id SERIAL PRIMARY KEY,
    admission_id INTEGER REFERENCES admissions(id),
    patient_id INTEGER REFERENCES patients(id),
    record_date DATE DEFAULT CURRENT_DATE,
    record_time TIME DEFAULT CURRENT_TIME,
    record_type VARCHAR(50) NOT NULL,
    vital_signs TEXT,
    consciousness VARCHAR(20),
    diet TEXT,
    intake_output TEXT,
    skin_condition TEXT,
    fall_risk VARCHAR(20),
    nursing_interventions TEXT,
    recorded_by INTEGER REFERENCES staff(id)
);

-- 护理计划表
CREATE TABLE nursing_plans (
    id SERIAL PRIMARY KEY,
    admission_id INTEGER REFERENCES admissions(id),
    patient_id INTEGER REFERENCES patients(id),
    plan_date DATE DEFAULT CURRENT_DATE,
    nursing_diagnosis TEXT,
    nursing_goals TEXT,
    nursing_interventions TEXT,
    evaluation_date DATE,
    evaluation TEXT,
    created_by INTEGER REFERENCES staff(id)
);

-- ----------------------------------------------------------------------------
-- Tables - 急诊管理
-- ----------------------------------------------------------------------------

-- 急诊分诊表
CREATE TABLE emergency_triage (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER REFERENCES patients(id),
    triage_date DATE DEFAULT CURRENT_DATE,
    triage_time TIME DEFAULT CURRENT_TIME,
    chief_complaint TEXT,
    vital_signs TEXT,
    pain_score INTEGER,
    triage_level priority_level NOT NULL,
    triage_nurse_id INTEGER REFERENCES staff(id),
    destination VARCHAR(50),
    notes TEXT
);

-- 急诊记录表
CREATE TABLE emergency_visits (
    id SERIAL PRIMARY KEY,
    visit_no VARCHAR(30) UNIQUE NOT NULL,
    patient_id INTEGER REFERENCES patients(id),
    triage_id INTEGER REFERENCES emergency_triage(id),
    arrival_date DATE DEFAULT CURRENT_DATE,
    arrival_time TIME DEFAULT CURRENT_TIME,
    chief_complaint TEXT,
    emergency_diagnosis TEXT,
    treatment_given TEXT,
    disposition VARCHAR(50),
    disposition_time TIME,
    disposition_destination VARCHAR(50),
    attending_physician_id INTEGER REFERENCES staff(id),
    status VARCHAR(20) DEFAULT 'completed'
);

-- ----------------------------------------------------------------------------
-- Indexes
-- ----------------------------------------------------------------------------

-- 基础信息索引
CREATE INDEX idx_staff_department ON staff(department_id);
CREATE INDEX idx_staff_type ON staff(staff_type);
CREATE INDEX idx_staff_status ON staff(status);
CREATE INDEX idx_departments_type ON departments(dept_type);
CREATE INDEX idx_schedules_staff ON schedules(staff_id);
CREATE INDEX idx_schedules_date ON schedules(schedule_date);

-- 患者相关索引
CREATE INDEX idx_patients_id_card ON patients(id_card);
CREATE INDEX idx_patients_phone ON patients(phone);
CREATE INDEX idx_patient_records_patient ON patient_records(patient_id);
CREATE INDEX idx_allergies_patient ON allergies(patient_id);
CREATE INDEX idx_medical_history_patient ON medical_history(patient_id);
CREATE INDEX idx_family_history_patient ON family_history(patient_id);

-- 门诊索引
CREATE INDEX idx_registrations_patient ON registrations(patient_id);
CREATE INDEX idx_registrations_doctor ON registrations(doctor_id);
CREATE INDEX idx_registrations_date ON registrations(registration_date);
CREATE INDEX idx_appointments_patient ON appointments(patient_id);
CREATE INDEX idx_appointments_doctor ON appointments(doctor_id);
CREATE INDEX idx_appointments_date ON appointments(appointment_date);
CREATE INDEX idx_outpatient_visits_patient ON outpatient_visits(patient_id);
CREATE INDEX idx_outpatient_visits_doctor ON outpatient_visits(doctor_id);
CREATE INDEX idx_outpatient_visits_date ON outpatient_visits(visit_date);
CREATE INDEX idx_diagnoses_patient ON diagnoses(patient_id);
CREATE INDEX idx_diagnoses_visit ON diagnoses(visit_id);

-- 住院索引
CREATE INDEX idx_wards_department ON wards(department_id);
CREATE INDEX idx_beds_ward ON beds(ward_id);
CREATE INDEX idx_beds_status ON beds(status);
CREATE INDEX idx_admissions_patient ON admissions(patient_id);
CREATE INDEX idx_admissions_department ON admissions(department_id);
CREATE INDEX idx_admissions_bed ON admissions(bed_id);
CREATE INDEX idx_admissions_dates ON admissions(admission_date, discharge_date);
CREATE INDEX idx_progress_notes_admission ON progress_notes(admission_id);
CREATE INDEX idx_medical_orders_admission ON medical_orders(admission_id);
CREATE INDEX idx_medical_orders_patient ON medical_orders(patient_id);

-- 检查检验索引
CREATE INDEX idx_exam_requests_patient ON examination_requests(patient_id);
CREATE INDEX idx_exam_requests_status ON examination_requests(status);
CREATE INDEX idx_exam_requests_date ON examination_requests(request_date);
CREATE INDEX idx_lab_requests_patient ON laboratory_requests(patient_id);
CREATE INDEX idx_lab_requests_status ON laboratory_requests(status);

-- 药房索引
CREATE INDEX idx_medicines_code ON medicines(medicine_code);
CREATE INDEX idx_medicines_type ON medicines(medicine_type);
CREATE INDEX idx_prescriptions_patient ON prescriptions(patient_id);
CREATE INDEX idx_prescriptions_status ON prescriptions(status);
CREATE INDEX idx_prescription_items_medicine ON prescription_items(medicine_id);
CREATE INDEX idx_medicine_inventory_medicine ON medicine_inventory(medicine_id);
CREATE INDEX idx_medicine_inventory_expiry ON medicine_inventory(expiry_date);

-- 手术索引
CREATE INDEX idx_surgery_requests_patient ON surgery_requests(patient_id);
CREATE INDEX idx_surgery_requests_status ON surgery_requests(status);
CREATE INDEX idx_surgery_requests_date ON surgery_requests(planned_date);

-- 费用索引
CREATE INDEX idx_charges_patient ON charges(patient_id);
CREATE INDEX idx_charges_visit ON charges(visit_id);
CREATE INDEX idx_charges_admission ON charges(admission_id);
CREATE INDEX idx_bills_patient ON bills(patient_id);
CREATE INDEX idx_bills_status ON bills(payment_status);
CREATE INDEX idx_payments_bill ON payments(bill_id);
CREATE INDEX idx_deposits_patient ON deposits(patient_id);
CREATE INDEX idx_deposits_admission ON deposits(admission_id);

-- 护理索引
CREATE INDEX idx_nursing_records_admission ON nursing_records(admission_id);
CREATE INDEX idx_nursing_records_patient ON nursing_records(patient_id);
CREATE INDEX idx_nursing_plans_admission ON nursing_plans(admission_id);

-- 急诊索引
CREATE INDEX idx_emergency_triage_patient ON emergency_triage(patient_id);
CREATE INDEX idx_emergency_triage_date ON emergency_triage(triage_date);
CREATE INDEX idx_emergency_visits_patient ON emergency_visits(patient_id);
CREATE INDEX idx_emergency_visits_date ON emergency_visits(arrival_date);

-- ----------------------------------------------------------------------------
-- Views (关键业务视图)
-- ----------------------------------------------------------------------------

-- 患者基本信息视图
CREATE VIEW v_patient_basic_info AS
SELECT
    p.id,
    p.patient_id,
    p.name,
    p.gender,
    p.birth_date,
    AGE(p.birth_date) AS age,
    p.phone,
    p.id_card,
    p.address,
    p.city,
    p.district,
    p.blood_type,
    p.insurance_company,
    p.insurance_policy_no,
    p.emergency_contact_name,
    p.emergency_contact_phone,
    p.registration_date,
    p.status
FROM patients p;

-- 今日挂号视图
CREATE VIEW v_today_registrations AS
SELECT
    r.id,
    r.registration_no,
    p.patient_id,
    p.name AS patient_name,
    p.phone AS patient_phone,
    d.dept_name,
    s.name AS doctor_name,
    r.registration_date,
    r.registration_time,
    r.registration_type,
    r.status,
    r.priority
FROM registrations r
JOIN patients p ON r.patient_id = p.id
JOIN departments d ON r.department_id = d.id
LEFT JOIN staff s ON r.doctor_id = s.id
WHERE r.registration_date = CURRENT_DATE;

-- 在院患者视图
CREATE VIEW v_current_inpatients AS
SELECT
    a.id,
    a.admission_no,
    p.patient_id,
    p.name AS patient_name,
    p.gender,
    p.age,
    d.dept_name,
    w.ward_name,
    b.bed_no,
    a.admission_date,
    CURRENT_DATE - a.admission_date AS days_in_hospital,
    a.diagnosis,
    a.nursing_level,
    s.name AS primary_doctor,
    n.name AS head_nurse,
    a.status
FROM admissions a
JOIN patients p ON a.patient_id = p.id
JOIN departments d ON a.department_id = d.id
JOIN wards w ON a.ward_id = w.id
JOIN beds b ON a.bed_id = b.id
LEFT JOIN staff s ON a.primary_doctor_id = s.id
LEFT JOIN staff n ON w.head_nurse_id = n.id
WHERE a.discharge_date IS NULL;

-- 医生今日排班视图
CREATE VIEW v_doctor_schedule_today AS
SELECT
    s.id AS staff_id,
    s.staff_id,
    s.name AS doctor_name,
    s.specialty,
    s.title,
    d.dept_name,
    sch.schedule_date,
    sch.shift_type,
    sch.start_time,
    sch.end_time,
    sch.location
FROM staff s
JOIN schedules sch ON s.id = sch.staff_id
JOIN departments d ON s.department_id = d.id
WHERE s.staff_type = 'doctor'
  AND sch.schedule_date = CURRENT_DATE
ORDER BY sch.start_time;

-- 待检查项目视图
CREATE VIEW v_pending_examinations AS
SELECT
    er.id,
    er.request_no,
    p.patient_id,
    p.name AS patient_name,
    ei.item_name,
    d.dept_name,
    er.request_date,
    er.priority,
    er.status,
    er.scheduled_date,
    er.scheduled_time,
    st.name AS requesting_doctor
FROM examination_requests er
JOIN patients p ON er.patient_id = p.id
JOIN examination_items ei ON er.item_id = ei.id
JOIN departments d ON ei.department_id = d.id
JOIN staff st ON er.requested_by = st.id
WHERE er.status IN ('ordered', 'scheduled')
ORDER BY er.priority, er.request_date;

-- 待检验项目视图
CREATE VIEW v_pending_laboratory_tests AS
SELECT
    lr.id,
    lr.request_no,
    p.patient_id,
    p.name AS patient_name,
    lt.test_name,
    lt.specimen_type,
    lr.request_date,
    lr.priority,
    lr.status,
    st.name AS requesting_doctor
FROM laboratory_requests lr
JOIN patients p ON lr.patient_id = p.id
JOIN laboratory_tests lt ON lr.test_id = lt.id
JOIN staff st ON lr.requested_by = st.id
WHERE lr.status IN ('collected', 'received', 'processing')
ORDER BY lr.priority, lr.request_date;

-- 待发药处方视图
CREATE VIEW v_pending_prescriptions AS
SELECT
    pr.id,
    pr.prescription_no,
    p.patient_id,
    p.name AS patient_name,
    st.name AS prescribing_doctor,
    pr.prescription_date,
    pr.diagnosis,
    COUNT(pi.id) AS item_count,
    pr.status
FROM prescriptions pr
JOIN patients p ON pr.patient_id = p.id
JOIN staff st ON pr.prescribed_by = st.id
LEFT JOIN prescription_items pi ON pr.id = pi.prescription_id
WHERE pr.status = 'pending'
GROUP BY pr.id, pr.prescription_no, p.patient_id, p.name, st.name, pr.prescription_date, pr.diagnosis, pr.status;

-- 患者费用汇总视图
CREATE VIEW v_patient_charges_summary AS
SELECT
    p.patient_id,
    p.name AS patient_name,
    COALESCE(c.visit_id, a.admission_no) AS encounter_no,
    COALESCE(c.charge_category, '住院') AS charge_category,
    COUNT(ch.id) AS charge_count,
    SUM(ch.total_amount) AS total_charges,
    MAX(ch.charge_date) AS last_charge_date
FROM patients p
LEFT JOIN charges ch ON p.id = ch.patient_id
LEFT JOIN charge_items c ON ch.charge_item_id = c.id
LEFT JOIN admissions a ON ch.admission_id = a.id
GROUP BY p.patient_id, p.name, c.visit_id, a.admission_no, c.charge_category;

-- 手术安排视图
CREATE VIEW v_surgery_schedule AS
SELECT
    sr.surgery_no,
    p.patient_id,
    p.name AS patient_name,
    sr.surgery_name,
    sr.planned_date,
    sr.planned_start_time,
    sr.planned_duration,
    oroom.room_no,
    sr.priority,
    sr.status,
    s1.name AS primary_surgeon,
    s2.name AS anesthesiologist,
    sr.anesthesia_type
FROM surgery_requests sr
JOIN patients p ON sr.patient_id = p.id
LEFT JOIN operating_rooms oroom ON sr.operating_room_id = oroom.id
LEFT JOIN staff s1 ON sr.primary_surgeon_id = s1.id
LEFT JOIN staff s2 ON sr.anesthesiologist_id = s2.id
WHERE sr.status IN ('scheduled', 'in_progress')
ORDER BY sr.planned_date, sr.planned_start_time;

-- 床位使用统计视图
CREATE VIEW v_bed_utilization AS
SELECT
    d.dept_name,
    w.ward_name,
    w.ward_no,
    COUNT(b.id) AS total_beds,
    COUNT(b.id) FILTER (WHERE b.status = 'occupied') AS occupied_beds,
    COUNT(b.id) FILTER (WHERE b.status = 'available') AS available_beds,
    COUNT(b.id) FILTER (WHERE b.status = 'maintenance') AS maintenance_beds,
    ROUND((COUNT(b.id) FILTER (WHERE b.status = 'occupied')::NUMERIC / COUNT(b.id) * 100), 2) AS occupancy_rate
FROM departments d
JOIN wards w ON d.id = w.department_id
JOIN beds b ON w.id = b.ward_id
GROUP BY d.dept_name, w.ward_name, w.ward_no
ORDER BY d.dept_name, w.ward_no;

-- 科室统计视图
CREATE VIEW v_department_statistics AS
SELECT
    d.id,
    d.dept_code,
    d.dept_name,
    d.dept_type,
    COUNT(DISTINCT s.id) FILTER (WHERE s.status = 'active') AS staff_count,
    COUNT(DISTINCT s.id) FILTER (WHERE s.staff_type = 'doctor' AND s.status = 'active') AS doctor_count,
    COUNT(DISTINCT s.id) FILTER (WHERE s.staff_type = 'nurse' AND s.status = 'active') AS nurse_count,
    COUNT(DISTINCT r.id) FILTER (WHERE r.registration_date = CURRENT_DATE) AS today_registrations,
    COUNT(DISTINCT a.id) FILTER (WHERE a.discharge_date IS NULL) AS current_inpatients,
    COALESCE(SUM(w.bed_count), 0) AS total_beds
FROM departments d
LEFT JOIN staff s ON d.id = s.department_id
LEFT JOIN wards w ON d.id = w.department_id
LEFT JOIN registrations r ON d.id = r.department_id
LEFT JOIN admissions a ON d.id = a.department_id
GROUP BY d.id, d.dept_code, d.dept_name, d.dept_type
ORDER BY d.dept_type, d.dept_name;

-- 药品库存预警视图
CREATE VIEW v_medicine_stock_alert AS
SELECT
    m.medicine_code,
    m.generic_name,
    SUM(mi.quantity) AS total_quantity,
    m.unit,
    MIN(mi.expiry_date) AS nearest_expiry_date,
    CASE
        WHEN SUM(mi.quantity) < 50 THEN '库存不足'
        WHEN MIN(mi.expiry_date) < CURRENT_DATE + 90 THEN '即将过期'
        ELSE '正常'
    END AS alert_type
FROM medicines m
JOIN medicine_inventory mi ON m.id = mi.medicine_id
GROUP BY m.id, m.medicine_code, m.generic_name, m.unit
HAVING SUM(mi.quantity) < 50 OR MIN(mi.expiry_date) < CURRENT_DATE + 90
ORDER BY nearest_expiry_date, total_quantity;

-- 急诊统计视图
CREATE VIEW v_emergency_statistics AS
SELECT
    ev.arrival_date,
    COUNT(*) AS total_visits,
    COUNT(*) FILTER (WHERE et.triage_level = 'emergency') AS emergency_count,
    COUNT(*) FILTER (WHERE et.triage_level = 'urgent') AS urgent_count,
    COUNT(*) FILTER (WHERE et.triage_level = 'high') AS high_count,
    COUNT(*) FILTER (WHERE et.triage_level = 'normal') AS normal_count,
    COUNT(*) FILTER (WHERE ev.disposition = 'admitted') AS admitted_count,
    COUNT(*) FILTER (WHERE ev.disposition = 'discharged') AS discharged_count
FROM emergency_visits ev
JOIN emergency_triage et ON ev.triage_id = et.id
WHERE ev.arrival_date >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY ev.arrival_date
ORDER BY ev.arrival_date DESC;

-- 患者诊断历史视图
CREATE VIEW v_patient_diagnosis_history AS
SELECT
    p.patient_id,
    p.name AS patient_name,
    d.diagnosis_code,
    d.diagnosis_name,
    d.diagnosis_type,
    d.diagnosis_date,
    s.name AS doctor_name,
    dept.dept_name
FROM patients p
JOIN diagnoses d ON p.id = d.patient_id
LEFT JOIN staff s ON d.diagnosed_by = s.id
LEFT JOIN departments dept ON s.department_id = dept.id
ORDER BY p.patient_id, d.diagnosis_date DESC;

-- ----------------------------------------------------------------------------
-- Sample Data
-- ----------------------------------------------------------------------------

-- 科室数据
INSERT INTO departments (dept_code, dept_name, dept_type, floor_no, description) VALUES
    ('OUT', '门诊部', 'clinical', 1, '门诊诊疗'),
    ('EMG', '急诊科', 'clinical', 1, '急诊急救'),
    ('INT', '内科', 'clinical', 2, '内科诊疗'),
    ('SUR', '外科', 'clinical', 3, '外科诊疗'),
    ('PED', '儿科', 'clinical', 4, '儿科诊疗'),
    ('OBG', '妇产科', 'clinical', 5, '妇产科诊疗'),
    ('ORT', '骨科', 'clinical', 3, '骨科诊疗'),
    ('CAR', '心血管科', 'clinical', 2, '心血管疾病'),
    ('NEU', '神经科', 'clinical', 6, '神经系统疾病'),
    ('RES', '呼吸科', 'clinical', 2, '呼吸系统疾病'),
    ('GAS', '消化科', 'clinical', 2, '消化系统疾病'),
    ('END', '内分泌科', 'clinical', 2, '内分泌疾病'),
    ('OPH', '眼科', 'clinical', 4, '眼科疾病'),
    ('ENT', '耳鼻喉科', 'clinical', 4, '耳鼻喉疾病'),
    ('DER', '皮肤科', 'clinical', 4, '皮肤疾病'),
    ('RAD', '放射科', 'medical_technical', -1, '影像检查'),
    ('LAB', '检验科', 'medical_technical', -1, '实验室检查'),
    ('PHA', '药房', 'logistics', 1, '药品管理'),
    ('ADM', '行政部', 'administrative', 0, '行政管理');

-- 员工数据
INSERT INTO staff (staff_id, name, gender, birth_date, phone, email, department_id, staff_type, title, specialty, hire_date) VALUES
    ('D001', '张建国', 'male', '1975-03-15', '13910001001', 'zhang.jg@hospital.com', 3, 'doctor', 'director', '心血管内科', '2005-03-01'),
    ('D002', '李秀英', 'female', '1978-06-20', '13910001002', 'li.xy@hospital.com', 3, 'doctor', 'chief', '消化内科', '2006-08-01'),
    ('D003', '王明华', 'male', '1980-09-10', '13910001003', 'wang.mh@hospital.com', 4, 'doctor', 'attending', '普通外科', '2008-09-01'),
    ('D004', '刘芳', 'female', '1982-12-05', '13910001004', 'liu.fang@hospital.com', 5, 'doctor', 'attending', '小儿内科', '2010-06-01'),
    ('D005', '陈强', 'male', '1976-04-18', '13910001005', 'chen.qiang@hospital.com', 7, 'doctor', 'chief', '创伤骨科', '2007-04-01'),
    ('D006', '赵丽', 'female', '1985-07-22', '13910001006', 'zhao.li@hospital.com', 6, 'doctor', 'attending', '产科', '2012-07-01'),
    ('D007', '孙伟', 'male', '1983-02-14', '13910001007', 'sun.wei@hospital.com', 9, 'doctor', 'attending', '神经内科', '2011-02-01'),
    ('D008', '周敏', 'female', '1984-05-30', '13910001008', 'zhou.min@hospital.com', 10, 'doctor', 'attending', '呼吸内科', '2011-05-01'),
    ('D009', '吴杰', 'male', '1979-08-08', '13910001009', 'wu.jie@hospital.com', 2, 'doctor', 'chief', '急诊医学', '2009-08-01'),
    ('D010', '郑丽', 'female', '1986-11-25', '13910001010', 'zheng.li@hospital.com', 13, 'doctor', 'attending', '眼科疾病', '2013-11-01'),
    ('N001', '冯涛', 'male', '1988-03-12', '13910002001', 'feng.tao@hospital.com', 3, 'nurse', NULL, NULL, '2014-03-01'),
    ('N002', '韩梅', 'female', '1990-06-18', '13910002002', 'han.mei@hospital.com', 4, 'nurse', NULL, NULL, '2015-06-01'),
    ('N003', '唐强', 'male', '1987-09-25', '13910002003', 'tang.qiang@hospital.com', 5, 'nurse', NULL, NULL, '2014-09-01'),
    ('N004', '董敏', 'female', '1992-01-08', '13910002004', 'dong.min@hospital.com', 7, 'nurse', NULL, NULL, '2016-01-01'),
    ('T001', '萧峰', 'male', '1989-04-15', '13910003001', 'xiao.feng@hospital.com', 16, 'technician', NULL, '医学影像', '2013-04-01'),
    ('T002', '于洁', 'female', '1991-07-20', '13910003002', 'yu.jie@hospital.com', 17, 'technician', NULL, '临床检验', '2015-07-01'),
    ('P001', '蒋伟', 'male', '1986-10-05', '13910004001', 'jiang.wei@hospital.com', 18, 'pharmacist', NULL, NULL, '2012-10-01'),
    ('A001', '蔡静', 'female', '1988-12-30', '13910005001', 'cai.jing@hospital.com', 19, 'admin', NULL, NULL, '2014-12-01');

UPDATE departments SET director_id = 1 WHERE dept_code = 'INT';
UPDATE departments SET director_id = 3 WHERE dept_code = 'SUR';
UPDATE departments SET director_id = 4 WHERE dept_code = 'PED';
UPDATE departments SET director_id = 5 WHERE dept_code = 'ORT';
UPDATE departments SET director_id = 9 WHERE dept_code = 'EMG';

-- 病房数据
INSERT INTO wards (ward_no, ward_name, department_id, floor_no, bed_count, nursing_level, charge_per_day) VALUES
    ('INT-01', '内科一病区', 3, 2, 45, 2, 300.00),
    ('INT-02', '内科二病区', 3, 2, 45, 2, 300.00),
    ('SUR-01', '外科一病区', 4, 3, 40, 2, 350.00),
    ('SUR-02', '外科二病区', 4, 3, 40, 2, 350.00),
    ('PED-01', '儿科病区', 5, 4, 30, 1, 250.00),
    ('ORT-01', '骨科病区', 7, 3, 35, 2, 320.00),
    ('OBG-01', '产科病区', 6, 5, 25, 1, 400.00),
    ('CAR-01', '心血管病区', 8, 2, 20, 3, 450.00),
    ('EMG-01', '急诊观察室', 2, 1, 15, 3, 280.00);

-- 病床数据
INSERT INTO beds (bed_no, ward_id, bed_type, charge_per_day) SELECT 'B' || i || '01', w.id, '普通', w.charge_per_day FROM wards w CROSS JOIN generate_series(1, 9) AS s(i);

-- 手术室数据
INSERT INTO operating_rooms (room_no, room_name, room_type, floor_no, capacity) VALUES
    ('OR-01', '第一手术室', '通用', -1, 8),
    ('OR-02', '第二手术室', '骨科', -1, 6),
    ('OR-03', '第三手术室', '微创', -1, 5),
    ('OR-04', '第四手术室', '急诊', -1, 4);

-- 检查项目数据
INSERT INTO examination_items (item_code, item_name, category, department_id, price, normal_duration) VALUES
    ('XRAY-CHEST', '胸部正位片', '放射', 16, 80.00, 30),
    ('CT-HEAD', '头颅CT平扫', 'CT', 16, 300.00, 20),
    ('CT-CHEST', '胸部CT平扫', 'CT', 16, 350.00, 20),
    ('MRI-BRAIN', '头颅MRI平扫', 'MRI', 16, 600.00, 30),
    ('US-ABDOMEN', '腹部彩超', '超声', 16, 150.00, 15),
    ('US-HEART', '心脏彩超', '超声', 16, 200.00, 20),
    ('ECG-Rest', '常规心电图', '心电', 16, 30.00, 10),
    ('ECG-Holter', '24小时动态心电图', '心电', 16, 180.00, 1440),
    ('ENDO-GASTRO', '胃镜检查', '内镜', 11, 250.00, 30),
    ('ENDO-COLON', '结肠镜检查', '内镜', 11, 300.00, 45);

-- 检验项目数据
INSERT INTO laboratory_tests (test_code, test_name, category, specimen_type, department_id, price, unit, turnaround_hours) VALUES
    ('CBC', '血常规', '血液学', '全血', 17, 25.00, NULL, 1),
    ('URINE', '尿常规', '尿液', '尿液', 17, 15.00, NULL, 1),
    ('GLU', '空腹血糖', '生化', '血清', 17, 10.00, 'mmol/L', 1),
    ('HBA1C', '糖化血红蛋白', '生化', '全血', 17, 50.00, '%', 4),
    ('LIPID', '血脂四项', '生化', '血清', 17, 60.00, NULL, 4),
    ('LIVER', '肝功能', '生化', '血清', 17, 80.00, NULL, 4),
    ('KIDNEY', '肾功能', '生化', '血清', 17, 50.00, NULL, 4),
    ('CRP', 'C反应蛋白', '免疫', '血清', 17, 30.00, 'mg/L', 2),
    ('TSH', '甲状腺功能', '内分泌', '血清', 17, 120.00, NULL, 8);

-- 药品数据
INSERT INTO medicines (medicine_code, generic_name, brand_name, medicine_type, category, specification, unit, manufacturer, price, prescription_required) VALUES
    ('M001', '阿莫西林胶囊', '阿莫灵', 'western', '抗生素', '0.25g*24粒', '盒', '制药A', 15.80, true),
    ('M002', '布洛芬缓释胶囊', '芬必得', 'western', '解热镇痛', '0.3g*20粒', '盒', '制药B', 22.00, true),
    ('M003', '奥美拉唑肠溶胶囊', '洛赛克', 'western', '消化系统', '20mg*14粒', '盒', '制药C', 35.00, true),
    ('M004', '阿托伐他汀钙片', '立普妥', 'western', '心血管', '20mg*7片', '盒', '制药D', 45.00, true),
    ('M005', '二甲双胍片', '格华止', 'western', '内分泌', '0.5g*20片', '盒', '制药E', 28.00, true),
    ('M006', '氨氯地平片', '络活喜', 'western', '心血管', '5mg*7片', '盒', '制药F', 32.00, true),
    ('M007', '感冒灵颗粒', '999', 'chinese', '感冒', '10g*9袋', '盒', '制药G', 12.00, false),
    ('M008', '连花清瘟胶囊', '以岭', 'chinese', '感冒', '0.35g*24粒', '盒', '制药H', 18.00, false),
    ('M009', '生理盐水', '0.9%氯化钠', 'western', '输液', '250ml', '袋', '制药I', 5.00, true),
    ('M010', '葡萄糖注射液', '5%葡萄糖', 'western', '输液', '250ml', '袋', '制药I', 6.00, true);

-- 患者数据
INSERT INTO patients (patient_id, name, gender, birth_date, phone, address, city, blood_type, emergency_contact_name, emergency_contact_phone) VALUES
    ('P20250001', '王子涵', 'male', '1985-03-15', '13810001001', '北京市朝阳区xx路xx号', '北京', 'A', '王军', '13810002001'),
    ('P20250002', '李欣怡', 'female', '1990-05-20', '13810001002', '北京市海淀区xx路xx号', '北京', 'B', '李明', '13810002002'),
    ('P20250003', '张浩然', 'male', '1978-07-08', '13810001003', '上海市浦东新区xx路xx号', '上海', 'O', '张伟', '13810002003'),
    ('P20250004', '陈思雨', 'female', '1995-09-12', '13810001004', '上海市徐汇区xx路xx号', '上海', 'AB', '陈强', '13810002004'),
    ('P20250005', '刘宇航', 'male', '1982-11-25', '13810001005', '广州市天河区xx路xx号', '广州', 'A', '刘洋', '13810002005'),
    ('P20250006', '杨梦琪', 'female', '1988-01-30', '13810001006', '深圳市南山区xx路xx号', '深圳', 'B', '杨涛', '13810002006'),
    ('P20250007', '赵子轩', 'male', '1992-04-18', '13810001007', '杭州市西湖区xx路xx号', '杭州', 'O', '赵明', '13810002007'),
    ('P20250008', '黄雨桐', 'female', '1987-06-22', '13810001008', '成都市武侯区xx路xx号', '成都', 'A', '黄磊', '13810002008'),
    ('P20250009', '周天宇', 'male', '1980-08-05', '13810001009', '武汉市洪山区xx路xx号', '武汉', 'AB', '周杰', '13810002009'),
    ('P20250010', '吴诗涵', 'female', '1993-10-10', '13810001010', '南京市鼓楼区xx路xx号', '南京', 'B', '吴峰', '13810002010'),
    ('P20250011', '徐浩然', 'male', '1975-02-14', '13810001011', '西安市雁塔区xx路xx号', '西安', 'O', '徐刚', '13810002011'),
    ('P20250012', '孙悦欣', 'female', '1989-05-28', '13810001012', '重庆市渝北区xx路xx号', '重庆', 'A', '孙健', '13810002012'),
    ('P20250013', '马子墨', 'male', '1996-07-16', '13810001013', '天津市和平区xx路xx号', '天津', 'B', '马超', '13810002013'),
    ('P20250014', '朱梓萱', 'female', '1984-09-20', '13810001014', '沈阳市沈河区xx路xx号', '沈阳', 'O', '朱伟', '13810002014'),
    ('P20250015', '胡峻熙', 'male', '1991-11-03', '13810001015', '苏州市工业园区xx路xx号', '苏州', 'AB', '胡涛', '13810002015'),
    ('P20250016', '郭梦瑶', 'female', '1986-01-08', '13810001016', '长沙市岳麓区xx路xx号', '长沙', 'A', '郭明', '13810002016'),
    ('P20250017', '何宇轩', 'male', '1979-03-22', '13810001017', '青岛市市南区xx路xx号', '青岛', 'B', '何强', '13810002017'),
    ('P20250018', '林诗雅', 'female', '1994-06-05', '13810001018', '大连市中山区xx路xx号', '大连', 'O', '林杰', '13810002018'),
    ('P20250019', '高梓豪', 'male', '1981-08-18', '13810001019', '厦门市思明区xx路xx号', '厦门', 'A', '高原', '13810002019'),
    ('P20250020', '罗雨欣', 'female', '1988-10-25', '13810001020', '宁波市海曙区xx路xx号', '宁波', 'B', '罗峰', '13810002020');

-- 挂号数据 (今日挂号)
INSERT INTO registrations (registration_no, patient_id, department_id, doctor_id, registration_type, priority, fee, status) SELECT
    'REG' || TO_CHAR(CURRENT_DATE, 'YYYYMMDD') || LPAD(ROW_NUMBER() OVER ()::TEXT, 4, '0'),
    p.id,
    CASE ROW_NUMBER() OVER () WHEN 1 THEN 3 WHEN 2 THEN 3 WHEN 3 THEN 4 WHEN 4 THEN 5 WHEN 5 THEN 7 WHEN 6 THEN 2 ELSE 3 END,
    CASE ROW_NUMBER() OVER () WHEN 1 THEN 1 WHEN 2 THEN 2 WHEN 3 THEN 3 WHEN 4 THEN 4 WHEN 5 THEN 5 WHEN 6 THEN 9 ELSE 1 END,
    'outpatient',
    CASE WHEN ROW_NUMBER() OVER () <= 2 THEN 'high' ELSE 'normal' END,
    15.00,
    'confirmed'
FROM (SELECT id FROM patients LIMIT 15) p;

-- 入院数据
INSERT INTO admissions (admission_no, patient_id, department_id, ward_id, bed_id, admission_date, admission_type, primary_doctor_id, diagnosis, nursing_level, deposit_amount, status) SELECT
    'ADM' || TO_CHAR(CURRENT_DATE, 'YYYYMMDD') || LPAD(ROW_NUMBER() OVER ()::TEXT, 3, '0'),
    p.id,
    3,
    1,
    b.id,
    CURRENT_DATE - (ROW_NUMBER() OVER () * 2 - 1)::INTEGER,
    '急诊入院',
    1,
    CASE ROW_NUMBER() OVER ()
        WHEN 1 THEN '急性心肌梗死'
        WHEN 2 THEN '急性胆囊炎'
        WHEN 3 THEN '社区获得性肺炎'
        WHEN 4 THEN '急性缺血性脑卒中'
        WHEN 5 THEN '2型糖尿病酮症酸中毒'
        ELSE '待查'
    END,
    CASE WHEN ROW_NUMBER() OVER () <= 2 THEN 'critical' WHEN ROW_NUMBER() OVER () <= 4 THEN 'grade_2' ELSE 'grade_3' END,
    5000.00,
    'admitted'
FROM (SELECT id FROM patients WHERE id <= 10) p
JOIN (SELECT id FROM beds WHERE id <= 10 ORDER BY id LIMIT 10) b ON p.id = b.id;

-- 门诊病历数据
INSERT INTO outpatient_visits (visit_no, patient_id, registration_id, department_id, doctor_id, chief_complaint, preliminary_diagnosis, status) SELECT
    'VIS' || TO_CHAR(CURRENT_DATE, 'YYYYMMDD') || LPAD(ROW_NUMBER() OVER ()::TEXT, 4, '0'),
    r.patient_id,
    r.id,
    r.department_id,
    r.doctor_id,
    CASE ROW_NUMBER() OVER ()
        WHEN 1 THEN '胸痛2小时，伴呼吸困难'
        WHEN 2 THEN '上腹部疼痛3天，加重1天'
        WHEN 3 THEN '发热、咳嗽1周'
        WHEN 4 THEN '头痛、眩晕2天'
        WHEN 5 THEN '多饮、多尿、体重下降1个月'
        ELSE '头晕乏力'
    END,
    CASE ROW_NUMBER() OVER ()
        WHEN 1 THEN '冠心病？'
        WHEN 2 THEN '急性胃炎'
        WHEN 3 THEN '上呼吸道感染'
        WHEN 4 THEN '高血压病'
        WHEN 5 THEN '糖尿病'
        ELSE '待查'
    END,
    'completed'
FROM registrations r WHERE r.status = 'confirmed' LIMIT 20;

-- 检查申请数据
INSERT INTO examination_requests (request_no, patient_id, visit_id, item_id, requested_by, priority, clinical_info) SELECT
    'EXR' || TO_CHAR(CURRENT_DATE, 'YYYYMMDD') || LPAD(ROW_NUMBER() OVER ()::TEXT, 4, '0'),
    v.patient_id,
    v.id,
    CASE ROW_NUMBER() OVER () WHEN 1 THEN 2 WHEN 2 THEN 3 WHEN 3 THEN 5 WHEN 4 THEN 6 WHEN 5 THEN 7 ELSE 1 END,
    v.doctor_id,
    CASE WHEN ROW_NUMBER() OVER () <= 3 THEN 'high' ELSE 'normal' END,
    '临床检查'
FROM outpatient_visits v LIMIT 25;

-- 检验申请数据
INSERT INTO laboratory_requests (request_no, patient_id, visit_id, test_id, requested_by, priority, clinical_diagnosis) SELECT
    'LBR' || TO_CHAR(CURRENT_DATE, 'YYYYMMDD') || LPAD(ROW_NUMBER() OVER ()::TEXT, 4, '0'),
    v.patient_id,
    v.id,
    CASE ROW_NUMBER() OVER () WHEN 1 THEN 1 WHEN 2 THEN 2 WHEN 3 THEN 3 WHEN 4 THEN 4 WHEN 5 THEN 5 ELSE 6 END,
    v.doctor_id,
    CASE WHEN ROW_NUMBER() OVER () <= 4 THEN 'high' ELSE 'normal' END,
    '常规检查'
FROM outpatient_visits v LIMIT 30;

-- 处方数据
INSERT INTO prescriptions (prescription_no, patient_id, visit_id, prescribed_by, diagnosis, status) SELECT
    'PRX' || TO_CHAR(CURRENT_DATE, 'YYYYMMDD') || LPAD(ROW_NUMBER() OVER ()::TEXT, 4, '0'),
    v.patient_id,
    v.id,
    v.doctor_id,
    v.preliminary_diagnosis,
    'pending'
FROM outpatient_visits v WHERE ROW_NUMBER() OVER () <= 15;

-- 药品库存数据
INSERT INTO medicine_inventory (medicine_id, batch_no, quantity, expiry_date, location) SELECT
    m.id,
    'BAT' || TO_CHAR(CURRENT_DATE, 'YYYYMMDD') || LPAD(ROW_NUMBER() OVER ()::TEXT, 3, '0'),
    CASE WHEN m.id <= 5 THEN 200 WHEN m.id <= 8 THEN 150 ELSE 500 END,
    CURRENT_DATE + (365 + (m.id * 30))::INTEGER,
    'A区' || m.id || '架'
FROM medicines m;

-- 费用项目数据
INSERT INTO charge_items (item_code, item_name, category, unit, price, department_id) VALUES
    ('REG-COMMON', '普通门诊挂号费', 'registration', '次', 15.00, 1),
    ('REG-EXPERT', '专家门诊挂号费', 'registration', '次', 50.00, 1),
    ('EXAM-XRAY', 'X线检查', 'examination', '次', 80.00, 16),
    ('EXAM-CT', 'CT检查', 'examination', '次', 300.00, 16),
    ('EXAM-MRI', 'MRI检查', 'examination', '次', 600.00, 16),
    ('LAB-BLOOD', '血液检查', 'laboratory', '次', 30.00, 17),
    ('MED-WEST', '西药费', 'medicine', '次', 1.00, 18),
    ('TREAT-INJ', '注射费', 'treatment', '次', 5.00, 3),
    ('BED-NORMAL', '普通床位费', 'hospitalization', '日', 300.00, 3),
    ('NUR-LEVEL2', '二级护理', 'nursing', '日', 50.00, 3);

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

    RAISE NOTICE '=== Large Hospital Information System Database Created ===';
    RAISE NOTICE 'Tables: %, Views: %, Indexes: %, Custom Types: %', table_count, view_count, index_count, type_count;
END $$;
