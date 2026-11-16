import sys

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
        # self._calculate_button.clicked.connect(self.extract_images)
        self._calculate_button.setEnabled(False)
        self._thread_pool = QtCore.QThreadPool.globalInstance()
        self._image_loader = None
        self._loaded_image_count = 0

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

    def create_folder_sizes_table(self):
        self._folder_sizes_box = QtWidgets.QGroupBox('Folder sizes')
        layout = QtWidgets.QVBoxLayout()
        self._table = QtWidgets.QTableView()
        data = pd.DataFrame(columns=['Folder', 'Size'])
        model = TableModel(data)
        self._table.setModel(model)
        layout.addWidget(self._table)
        self._folder_sizes_box.setLayout(layout)


if __name__ == '__main__':
    app = QtWidgets.QApplication([])

    widget = MyWidget()
    widget.showMaximized()
    widget.show()

    sys.exit(app.exec())
