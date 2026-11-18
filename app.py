import sys
import time
from pathlib import Path

import pandas as pd
from PySide6 import QtCore, QtWidgets


class TableModel(QtCore.QAbstractTableModel):
    def __init__(self, data):
        super().__init__()
        self._data = data

    def data(self, index, role):
        if role == QtCore.Qt.DisplayRole:
            value = self._data.iloc[index.row(), index.column()]
            return str(value)

    def rowCount(self, index):
        return self._data.shape[0]

    def columnCount(self, index):
        return self._data.shape[1]

    def headerData(self, section, orientation, role):
        # section is the index of the column/row.
        if role == QtCore.Qt.DisplayRole:
            if orientation == QtCore.Qt.Horizontal:
                return str(self._data.columns[section])

            if orientation == QtCore.Qt.Vertical:
                return str(self._data.index[section])


class SizeCalculatorSignals(QtCore.QObject):
    size_calculated = QtCore.Signal(str, object)
    finished = QtCore.Signal()


class SizeCalculator(QtCore.QRunnable):
    def __init__(self, folder_path):
        super().__init__()
        self.folder_path = Path(folder_path)
        self.signals = SizeCalculatorSignals()

    @QtCore.Slot()
    def run(self):
        def calculate_entry_size(entry):
            try:
                if entry.is_file():
                    return entry.stat().st_size

                folder_path = str(entry.resolve())
                folder_size = sum(
                    calculate_entry_size(child_entry) for child_entry in entry.iterdir()
                )
                if folder_size > 1e9:
                    self.signals.size_calculated.emit(folder_path, folder_size)
                return folder_size
            except Exception as _:
                return 0

        try:
            calculate_entry_size(self.folder_path)
        finally:
            self.signals.finished.emit()


class MyWidget(QtWidgets.QWidget):
    def __init__(self):
        super().__init__()

        self.create_folder_upload_box()
        self.create_folder_sizes_table()

        self.layout = QtWidgets.QVBoxLayout(self)
        self.layout.addWidget(self._folder_upload_box)
        self.layout.addWidget(self._folder_sizes_box)

    def create_folder_upload_box(self):
        self._folder_upload_box = QtWidgets.QGroupBox('Folder upload')
        layout = QtWidgets.QHBoxLayout()

        self._folder_path = QtWidgets.QLineEdit(readOnly=True)
        self._browse_button = QtWidgets.QPushButton('Browse')
        self._calculate_button = QtWidgets.QPushButton('Calculate Size')

        self._browse_button.clicked.connect(self.open_folder_dialog)
        self._calculate_button.clicked.connect(self.calculate_folder_sizes)
        self._calculate_button.setEnabled(False)
        self._thread_pool = QtCore.QThreadPool.globalInstance()
        self._size_calculator = None

        layout.addWidget(QtWidgets.QLabel('Folder'))
        layout.addWidget(self._folder_path)
        layout.addWidget(self._browse_button)
        layout.addWidget(self._calculate_button)

        self._folder_upload_box.setLayout(layout)

    def open_folder_dialog(self):
        self._dir = QtWidgets.QFileDialog.getExistingDirectory()

        if self._dir:
            self._folder_path.setText(self._dir)
            self._calculate_button.setEnabled(True)

    def calculate_folder_sizes(self):
        self._start_time = time.perf_counter()
        self._browse_button.setEnabled(False)
        self._calculate_button.setEnabled(False)
        self._size_calculator = SizeCalculator(self._dir)
        self._size_calculator.signals.size_calculated.connect(self.add_calculated_size)
        self._size_calculator.signals.finished.connect(self.finish_calculating_sizes)
        self._thread_pool.start(self._size_calculator)

    @QtCore.Slot(str, object)
    def add_calculated_size(self, folder_path, size):
        self._data.loc[len(self._data)] = [folder_path, size / 1e9]
        self._data = self._data.sort_values(by='Size (GB)', ascending=False, ignore_index=True)
        self._model._data = self._data
        self._model.layoutChanged.emit()

    @QtCore.Slot()
    def finish_calculating_sizes(self):
        self._browse_button.setEnabled(True)
        self._calculate_button.setEnabled(True)
        self._size_calculator = None
        print(f'Execution time: {time.perf_counter() - self._start_time} s')

    def create_folder_sizes_table(self):
        self._folder_sizes_box = QtWidgets.QGroupBox('Folder sizes')
        layout = QtWidgets.QVBoxLayout()

        self._table = QtWidgets.QTableView()
        self._data = pd.DataFrame(columns=['Folder', 'Size (GB)'])
        self._model = TableModel(self._data)
        self._table.setModel(self._model)

        layout.addWidget(self._table)
        self._folder_sizes_box.setLayout(layout)

        header = self._table.horizontalHeader()
        header.setSectionResizeMode(QtWidgets.QHeaderView.ResizeMode.ResizeToContents)
        header.setSectionResizeMode(1, QtWidgets.QHeaderView.ResizeMode.ResizeToContents)
        header.setSectionResizeMode(0, QtWidgets.QHeaderView.ResizeMode.Stretch)


if __name__ == '__main__':
    app = QtWidgets.QApplication([])

    widget = MyWidget()
    widget.showMaximized()
    widget.show()

    sys.exit(app.exec())
