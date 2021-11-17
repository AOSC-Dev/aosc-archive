import os
import re
import shutil
import sys
import hashlib
#将退休的deb包存放到内存限制为25G的文件夹中
savedStdout = sys.stdout  #保存标准输出流

path = '/home/s/archive/main' #旧包路径
opath= '/home/s/retire/main' #初始deb包的路径
filelist=[]
for root, dir, files in os.walk(path):#生成所有旧包的列表

    for file in files:
       filelist.append(file)
filelist.sort()

def GetFileDictpath(dir, fileDict):
    newDir = dir
    if os.path.isfile(dir):  # 如果是文件则添加进 fileDict
        fileDict[os.path.basename(dir)]=dir
    elif os.path.isdir(dir):
        for s in os.listdir(dir):  # 如果是文件夹
            newDir = os.path.join(dir, s)
            GetFileDictpath(newDir, fileDict)
    return fileDict

filedict={}
GetFileDictpath(path,filedict)
file_abspath={}
for key in filedict:
    str_path=filedict[key].split(sep='/')[-2]
    file_abspath[key]=str_path

def get_doc_real_size(p_doc):#返回文件夹大小
    size = 0.0
    for root, dirs, files in os.walk(p_doc):
        size += sum([os.path.getsize(os.path.join(root, file)) for file in files])
    size = round(size/1024/1024/1024, 2)
    return size

def del_empty_file(path):#删除空文件夹
    # 获取当前目录下的所有文件夹名称  得到的是一个列表
    folders=os.listdir(path)
    for folder in folders:
        # 将上级路径path与文件夹名称folder拼接出文件夹的路径
        folder2=os.listdir(path+'/'+folder)
    # 若文件夹为空
        if folder2==[]:
            # 并将此空文件夹删除
            os.rmdir(path+'/'+folder)

file_memory={}
for file in filelist:#生成文件内存
    file_memory[file]=os.stat(filedict[file]).st_size/1024/1024/1024
n=0
os.makedirs(path + '/old_deb' + str(n))

for i in os.listdir(opath):
    os.makedirs(path + '/old_deb' + str(n) + '/' + i)

os.chdir(path)
for file in filelist:#生成文件内存限制为25G的文件夹
        if get_doc_real_size(path+'/old_deb'+str(n))+file_memory[file]<23:
                shutil.move(filedict[file], path+'/old_deb'+str(n)+ '/' +file_abspath[file]+'/'+file)
        else:
                del_empty_file(path+'/old_deb'+str(n))
                print_log = open(path + '/old_deb' + str(n) + '/deb_name', 'a')
                sys.stdout = print_log
                print(os.popen('find ' + 'old_deb' + str(n) + ' -type f').read())
                n=n+1
                if os.path.exists(path + '/old_deb' + str(n)) == False:
                    os.makedirs(path + '/old_deb' + str(n))
                    for i in os.listdir(opath):
                        os.makedirs(path + '/old_deb' + str(n) + '/' + i)
                    shutil.move(filedict[file], path + '/old_deb' + str(n) + '/' + file_abspath[file] + '/' + file)


del_empty_file(path+'/old_deb'+str(n))
print_log = open(path+'/old_deb'+str(n)+ '/deb_name', 'a')
sys.stdout = print_log
print(os.popen('find '+'old_deb'+str(n)+ ' -type f').read())

del_empty_file(path)
sys.stdout = savedStdout
