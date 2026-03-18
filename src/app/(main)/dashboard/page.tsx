export default function DashboardPage() {
  return (
    <div>
      <h1 className="text-3xl font-bold mb-6">Dashboard</h1>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="card bg-base-200 shadow">
          <div className="card-body">
            <h2 className="card-title">Provas</h2>
            <p className="text-4xl font-bold text-primary">-</p>
          </div>
        </div>
        <div className="card bg-base-200 shadow">
          <div className="card-body">
            <h2 className="card-title">Alunos</h2>
            <p className="text-4xl font-bold text-primary">-</p>
          </div>
        </div>
        <div className="card bg-base-200 shadow">
          <div className="card-body">
            <h2 className="card-title">Matérias</h2>
            <p className="text-4xl font-bold text-primary">-</p>
          </div>
        </div>
      </div>
    </div>
  );
}
